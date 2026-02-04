use axum::{Router, routing::get};
use dotenv::dotenv;
use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;
use tokio::net::TcpListener;

use crate::utils::log_error;

mod bot;
mod commands;
mod data;
mod handlers;
mod utils;

#[tokio::main]
async fn main() {
    /* Create data.json */

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
            println!("'{}' created!", path);
        }
    }

    /* Start Bot */

    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Missing token in .env");

    tokio::spawn(async {
        bot::run(token).await;
    });

    /* For Web Service Hosting */

    let app = Router::new().route("/", get(|| async { "📊 CountLogger Online" }));

    let port = std::env::var("PORT").unwrap_or("3000".to_string());

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
