use crate::data::BotData;
use crate::utils::internal_err;
use serenity::builder::*;
use serenity::model::application::*;
use serenity::prelude::*;

pub fn register() -> CreateCommand {
    CreateCommand::new("help").description("Full guide about this bot")
}

pub async fn execute(ctx: Context, command: CommandInteraction, bot_data: &BotData) {
    if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(r#"A bot that can log progress of a counting channel in your guild!

## **-USAGE-**

Setup and let it run, then it will logs counting history automatically.

## **-COMMAND-**

`/help` : Full guide about this bot
`/ping` : Shows basic stats

### **(Admin only)**
`/setup info` : Show info of current setup
`/setup reset` : Reset entire current setup
`/setup channels` `[log_channel]` `[counting_channel]` : Set each specified channel as current
`/setup timezone` `[utc]` : Set timezone for logging clarity
`/setup language` `[lang]` `[lang2: OPTIONAL]` : Set language for logging clarity
`/relog` : Refresh and update all logs from the start

## **-FORMAT-**

**📊 Year `<year> (<iteration>)` Count Log**
`Date : Sum (<update_interval>)`
`<MM/DD> : <total_count> (+<count>)`

## **-RECOMMENDED-**

- Please use this bot alongside actual counting checking bot like `Countr` or others, as this bot only purpose is to log counting history.
- Setup `[log_channel]` on an empty channel. It is dangerous to set this on a channel with message history.
- Do `/relog` to refresh and see changes everytime after done `/setup` new configurations.
"#)
            .flags(InteractionResponseFlags::EPHEMERAL)
    )).await {
        internal_err(&ctx, &command, &e.to_string()).await;
    }
}
