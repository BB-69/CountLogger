use crate::data::{BotData, BotDataKey};
use crate::handlers;
use crate::utils::{log_error, log_info, log_warn};
use chrono::Utc;
use serenity::Client;
use serenity::all::GatewayIntents;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub async fn run(token: String, pool: Pool<Postgres>) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        log_info("🎧 Creating Discord client…");

        let bot_start = Utc::now();

        let bot_data = Arc::new(BotData {
            pool: Arc::new(pool.clone()),
            start_time: bot_start,
        });

        let handler = handlers::Handler::new(bot_data.clone());

        let intents = GatewayIntents::GUILDS
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let mut client = Client::builder(&token, intents)
            .event_handler(handler)
            .await
            .expect("Error creating client");

        {
            client.data.write().await.insert::<BotDataKey>(bot_data);
            log_info("✅ Bot startup complete, connecting to gateway…");
        }

        if let Err(e) = client.start().await {
            log_error(&format!("❌ Discord gateway exited: {e}"));
        } else {
            log_warn("⚠️ Discord client exited without error");
        }

        log_info("🔁 Reconnecting in 10 seconds…");
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}
