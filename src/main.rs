use axum::{Router, routing::get};
use dotenv::dotenv;
use std::env;
use tokio::net::TcpListener;

mod bot;
mod commands;
mod data;
mod handlers;
mod utils;

#[tokio::main]
async fn main() {
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
