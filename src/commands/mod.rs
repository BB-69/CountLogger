use serenity::all::*;
use crate::data::BotData;

pub mod ping;

pub async fn handle(ctx: Context, command: CommandInteraction, bot_data: &BotData) {
    match command.data.name.as_str() { /*---handle every commands here---*/
        "ping" => ping::execute(ctx, command, bot_data).await, _ => {
            let _ = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("❓ Unknown command")
            )).await;
        }
    }
}
