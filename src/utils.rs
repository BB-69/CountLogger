use serenity::prelude::*;
use serenity::model::application::*;
use serenity::builder::*;

pub fn log_info(msg: &str) {
    println!("[INFO]: {}", msg);
}

pub fn log_error(msg: &str) {
    eprintln!("[ERROR]: {}", msg);
}

pub async fn check_admin(ctx: &Context, command: &CommandInteraction) -> bool {
    let member = command.member.as_ref().unwrap();

    if member.permissions.unwrap_or_default().administrator() { return true; }

    let _ = command.create_response(&ctx.http, CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content("⛔ You need admin privileges to use this command!")
            .flags(InteractionResponseFlags::EPHEMERAL)
    )).await;

    false
}