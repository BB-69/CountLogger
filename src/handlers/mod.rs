pub mod ready;

use crate::data::{BotData, load_guild_data, save_guild_data};
use chrono::*;
use serenity::all::{ActivityData, Message};
use serenity::async_trait;
use serenity::model::prelude::Interaction;
use serenity::prelude::*;

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

        ctx.set_activity(Some(ActivityData::playing(
            "In Counting Channel With You! 💙",
        )));

        let commands = serenity::model::application::Command::set_global_commands(
            &ctx.http,
            vec![
                /*---register every commands here---*/
                crate::commands::ping::register(),
                crate::commands::help::register(),
                crate::commands::setup::register(),
                crate::commands::relog::register(),
                crate::commands::message::register(),
            ],
        )
        .await;

        if let Err(e) = commands {
            crate::utils::log_error(&format!("Failed to register commands: {:?}", e));
        }

        // Others
        let bot_data = std::sync::Arc::clone(&self.bot_data);
        tokio::spawn(crate::commands::relog::log_daily_counts(ctx, bot_data));
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            crate::commands::handle(ctx, command, &self.bot_data).await;
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if let Some(guild_id) = msg.guild_id {
            let mut modified = false;
            let guild_id_u64 = guild_id.get();
            match load_guild_data(&self.bot_data.pool, guild_id_u64).await {
                Ok(mut guild_data) => {
                    if let Some(_count_ch_id) = guild_data.ids.counting_channel_id {
                        if msg.content.parse::<i64>().is_ok() {
                            let key = get_current_time(guild_data.settings.utc);
                            if let Ok(num) = msg.content.parse::<i64>() {
                                guild_data.daily_counts.insert(key, num);
                                modified = true;
                            }
                        }
                    }

                    if !modified {
                        return;
                    }
                    let _ = save_guild_data(&self.bot_data.pool, guild_id_u64, &guild_data).await;
                }
                Err(e) => eprintln!("❌ Cannot load data from Guild{guild_id_u64}: {e}"),
            }
        }
    }
}

fn get_current_time(utc: i8) -> String {
    let utc_now: DateTime<Utc> = Utc::now();
    let offset_now =
        utc_now.with_timezone(&chrono::FixedOffset::east_opt(utc as i32 * 3600).unwrap());
    offset_now.format("%Y-%m-%d").to_string()
}
