use crate::data::{BotData, BotDataKey, load_all_data};
use crate::handlers;
use chrono::Utc;
use serenity::Client;
use serenity::all::GatewayIntents;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn run(token: String) {
    let all_data = load_all_data();
    let guilds_map = Arc::new(Mutex::new(all_data.0));
    let bot_start = Utc::now();
    let bot_data = Arc::new(BotData {
        guilds: guilds_map,
        start_time: bot_start,
    });
    let handler = handlers::Handler::new(bot_data.clone());

    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    {
        client.data.write().await.insert::<BotDataKey>(bot_data);
        crate::utils::log_info("Bot startup complete!");
    }

    if let Err(e) = client.start().await {
        crate::utils::log_error(&format!("Client error: {:?}", e));
    }
}
