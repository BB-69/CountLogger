use dotenv::dotenv;
use std::env;

mod bot;
mod commands;
mod data;
mod handlers;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Missing token in .env");

    bot::run(token).await;
}
