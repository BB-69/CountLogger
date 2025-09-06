use serenity::prelude::*;
use serenity::model::application::*;
use serenity::builder::*;

use crate::data::{BotData, load_guild_data, save_guild_data};
use crate::utils::check_admin;

pub fn register() -> CreateCommand {
    CreateCommand::new("help")
        .description("Full guide about this bot")
}

pub async fn execute(ctx: Context, command: CommandInteraction, bot_data: &BotData) {
    let _ = command.create_response(&ctx.http, CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(r#"A bot that can log progress of a counting channel in your guild!

## **-USAGE-**
Let it run and it will automatically update your logs every `5 minutes`

## **-COMMAND-**
`/help` : Full guide about this bot

### **(Admin only)**
`/setup info` : View your current channel set up
`/setup <your_log_channel> <your_counting_channel>` : Set each specified channel as current
`/relog` : Recalculate and update all count logs in `<your_log_channel>`

## **-FORMAT-**
**📊 Year `<year> (iteration)` Count Log:**
`日にち/date : 合計/sum  (5minutes change)`
`YYYY/MM/DD` : `<total_count> (<count>)`

### **-DISCLAIMER-**
This will currently detect last number of each day in `<your_counting_channel>` regardless of order.
We recommend using another helping bot with proper counting rules checking alongside this one."#)
            .flags(InteractionResponseFlags::EPHEMERAL)
    )).await;
}
