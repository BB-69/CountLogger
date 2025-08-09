use serenity::Client;
use serenity::all::GatewayIntents;
use crate::handlers;
use crate::data::{BotData, BotDataKey};
use std::sync::Arc;

pub async fn run(token: String) {
    let bot_data = Arc::new(BotData::default());
    let handler = handlers::Handler::new(bot_data.clone());

    let intents = GatewayIntents::GUILDS;
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
