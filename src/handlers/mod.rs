pub mod ready;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::prelude::Interaction;
use crate::data::BotData;

pub struct Handler {
    pub bot_data: std::sync::Arc<BotData>,
}

impl Handler {
    pub fn new(bot_data: std::sync::Arc<BotData>) -> Self {
        Self { bot_data }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: serenity::prelude::Context, ready: serenity::model::gateway::Ready) {
        self::ready::on_ready(&ctx, &ready).await;

        let commands = serenity::model::application::Command::set_global_commands(
            &ctx.http,
            vec![ /*---register every commands here---*/
                crate::commands::ping::register(),
            ],
        ).await;

        if let Err(e) = commands {
            crate::utils::log_error(&format!("Failed to register commands: {:?}", e));
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            crate::commands::handle(ctx, command, &self.bot_data).await;
        }
    }
}
