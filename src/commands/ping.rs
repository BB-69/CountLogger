use crate::{data::BotData, utils::internal_err};
use serenity::builder::*;
use serenity::model::application::*;
use serenity::prelude::*;
use std::time::Instant;

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("🏓Pong! Shows basic stats")
}

pub async fn execute(ctx: Context, command: CommandInteraction, bot_data: &BotData) {
    let start = Instant::now();

    // calculate latency
    let latency_ms = start.elapsed().as_millis();

    // basic info
    let servers_count = bot_data.guilds.lock().await.len();
    let uptime = humantime::format_duration(
        chrono::Utc::now()
            .signed_duration_since(bot_data.start_time)
            .to_std()
            .unwrap_or_default(),
    )
    .to_string();

    let reply = format!(
        "🏓 Pong!\nLatency: `{}ms`\nServers: `{}`\nUptime: `{}`",
        latency_ms, servers_count, uptime
    );

    if let Err(e) = command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(reply)
                    .flags(InteractionResponseFlags::EPHEMERAL),
            ),
        )
        .await
    {
        internal_err(&ctx, &command, &e.to_string()).await;
    }
}
