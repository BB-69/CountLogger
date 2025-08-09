use serenity::prelude::*;
use serenity::model::application::*;
use serenity::builder::*;

use crate::data::{BotData, load_guild_data, save_guild_data};
use crate::utils::check_admin;

pub fn register() -> CreateCommand {
    CreateCommand::new("ping")
        .description("Replies with Pong! (Admins only test)")
}

pub async fn execute(ctx: Context, command: CommandInteraction, bot_data: &BotData) {
    if !check_admin(&ctx, &command).await { return; }

    if let Some(guild_id) = command.guild_id {
        let guild_id_u64 = guild_id.get();

        /*---( Access & Modify data here)---*/

        let mut guild_data = load_guild_data(guild_id_u64);
        {
            let mut guilds = bot_data.guilds.lock().await;
            guilds.insert(guild_id_u64, guild_data.clone());
        }
        save_guild_data(guild_id_u64, &guild_data);
    }

    let _ = command.create_response(&ctx.http, CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content("🏓 Pong!")
            .flags(InteractionResponseFlags::EPHEMERAL)
    )).await;
}
