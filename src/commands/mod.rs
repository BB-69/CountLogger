use std::collections::HashMap;
use serenity::all::*;
use crate::data::BotData;

pub mod ping;
pub mod help;
pub mod setup;
pub mod relog;

macro_rules! command_map {
    ( $( $name:literal => $module:ident ),* $(,)? ) => {{
        use std::collections::HashMap;
        use std::future::Future;
        use std::pin::Pin;
        use std::sync::Arc;

        type CommandHandler = Arc<
            dyn for<'a> Fn(
                Context,
                CommandInteraction,
                &'a BotData,
            ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>
            + Send
            + Sync,
        >;

        let mut map: HashMap<&'static str, CommandHandler> = HashMap::new();

        $(
            map.insert($name, Arc::new(|ctx, cmd, bot_data| {
                Box::pin($module::execute(ctx, cmd, bot_data))
            }));
        )*

        map
    }};
}

pub async fn handle(ctx: Context, command: CommandInteraction, bot_data: &BotData) {
    let commands = command_map! {
        /*---register every commands here---*/
        "ping" => ping,
        "help" => help,
        "setup" => setup,
        "relog" => relog,
    };

    if let Some(handler) = commands.get(command.data.name.as_str()) {
        handler(ctx, command, bot_data).await;
    } else {
        let _ = command.create_response(&ctx.http, CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("❓ Unknown command")
        )).await;
    }
}
