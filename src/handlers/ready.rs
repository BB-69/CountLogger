use crate::utils::log_info;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

pub async fn on_ready(_ctx: &Context, ready: &Ready) {
    log_info(&format!("✅ Bot is online as {}", ready.user.name));
}
