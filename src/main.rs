use axum::{Router, routing::get};
use dotenv::dotenv;
use std::fs;
use std::path::Path;
use std::process::exit;
use std::{env, time::Duration};
use tokio::net::TcpListener;

use crate::utils::log_error;

mod bot;
mod commands;
mod data;
mod handlers;
mod utils;

#[tokio::main]
async fn main() {
    println!("🚀 App booted at {:?}", std::time::SystemTime::now());

    // ===== DATA JSON =====

    {
        let path = "src/data/data.json";
        let path_obj = Path::new(path);

        if !path_obj.exists() {
            if let Some(parent) = path_obj.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    log_error(&e.to_string());
                    exit(1);
                }
            }

            if let Err(e) = fs::write(path, "{}") {
                log_error(&e.to_string());
                exit(1);
            }
            println!("📄 '{}' created!", path);
        }
    }

    dotenv().ok();

    // ===== ENV CHECKS =====
    let token = env::var("DISCORD_TOKEN").expect("❌ DISCORD_TOKEN missing from environment");

    let port = env::var("PORT").unwrap_or_else(|_| {
        println!("⚠️ PORT not set, defaulting to 3000");
        "3000".to_string()
    });

    println!("🔑 Discord token loaded");
    println!("🌐 Web server will bind to port {port}");

    // ===== BOT SUPERVISOR TASK =====
    let bot_task = tokio::spawn(async move {
        loop {
            println!("🎧 Starting Discord bot…");

            if let Err(e) = bot::run(token.clone()).await {
                eprintln!("❌ Discord bot crashed: {e}");
            } else {
                eprintln!("⚠️ Discord bot exited without error (unexpected)");
            }

            println!("🔁 Restarting Discord bot in 5 seconds…");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    // ===== WEB SERVER =====
    let app = Router::new()
        .route("/", get(|| async { "📊 CountLogger Online 💙" }))
        .route("/health", get(|| async { "ok" }));

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr)
        .await
        .expect("❌ Failed to bind TCP listener");

    println!("✅ Web server listening on http://{addr}");

    let web_task = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("❌ Web server crashed: {e}");
        }
    });

    // ===== SUPERVISOR =====
    tokio::select! {
        _ = bot_task => {
            eprintln!("💀 Bot supervisor task ended (this should NEVER happen)");
        }
        _ = web_task => {
            eprintln!("💀 Web server task ended");
        }
    }
}
