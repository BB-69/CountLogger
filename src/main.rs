use axum::{Router, routing::get};
use dotenv::dotenv;
use std::env;
// use std::fs;
// use std::path::Path;
// use std::process::exit;
use tokio::net::TcpListener;

// use crate::utils::log_error;

mod bot;
mod commands;
mod data;
mod handlers;
mod utils;

#[tokio::main]
async fn main() {
    env_logger::init();
    println!("🚀 App booted at {:?}", std::time::SystemTime::now());

    // ===== DATA JSON =====

    // {
    //     let path = "src/data/data.json";
    //     let path_obj = Path::new(path);

    //     if !path_obj.exists() {
    //         if let Some(parent) = path_obj.parent() {
    //             if let Err(e) = fs::create_dir_all(parent) {
    //                 log_error(&e.to_string());
    //                 exit(1);
    //             }
    //         }

    //         if let Err(e) = fs::write(path, "{}") {
    //             log_error(&e.to_string());
    //             exit(1);
    //         }
    //         println!("📄 '{}' created!", path);
    //     }
    // }

    dotenv().ok();

    // ===== ENV CHECKS =====
    let token = env::var("DISCORD_TOKEN").expect("❌ DISCORD_TOKEN missing");

    let port = env::var("PORT").unwrap_or_else(|_| {
        println!("⚠️ PORT not set, defaulting to 3000");
        "3000".to_string()
    });

    println!("🔑 Discord token loaded");
    println!("🌐 Web server port: {port}");

    let database_url = std::env::var("DATABASE_URL").expect("❌ DATABASE_URL not set");

    let (tx, rx) = tokio::sync::oneshot::channel();

    // ===== DATABASE =====
    tokio::spawn(async move {
        'outer: loop {
            match sqlx::PgPool::connect(&database_url).await {
                Err(e) => eprintln!("❌ Couldn't connect to Database: {e}"),
                Ok(pool) => {
                    let row: (i64,) = sqlx::query_as("select 1::bigint")
                        .fetch_one(&pool)
                        .await
                        .unwrap();

                    println!("✅ DB OK: {:?}", row);

                    sqlx::query("select * from public.guilds limit 1")
                        .execute(&pool)
                        .await
                        .unwrap();

                    tx.send(pool).unwrap();
                    break 'outer;
                }
            }

            println!("🔁 Trying Database again in 10 seconds…");
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    });

    // ===== DISCORD BOT =====
    tokio::spawn(async move {
        let pool = rx.await.unwrap();
        if let Err(e) = bot::run(token, pool).await {
            eprintln!("💀 Bot task exited unexpectedly: {e}");
        }
    });

    // ===== WEB SERVER (Render keep-alive) =====
    let app = Router::new()
        .route("/", get(|| async { "📊 CountLogger Online 💙" }))
        .route("/health", get(|| async { "ok" }));

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr)
        .await
        .expect("❌ Failed to bind TCP listener");

    println!("✅ Web server listening on http://{addr}");

    // This should NEVER exit
    axum::serve(listener, app)
        .await
        .expect("❌ Axum server crashed");
}
