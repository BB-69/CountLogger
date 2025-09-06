use serenity::all::{ChannelId, ChannelType, Guild};
use serenity::prelude::*;
use serenity::model::application::*;
use serenity::builder::*;

use crate::data::structs::GuildData;
use crate::data::{BotData, load_guild_data, save_guild_data};
use crate::utils::*;

pub fn register() -> CreateCommand {
    CreateCommand::new("setup")
        .description("Setup bundled commands")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "info",
                "Show info of current setup",
            )
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "reset",
                "Reset entire current setup",
            )
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "channels",
                "Set log and counting channels",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "log_channel",
                    "Channel for logs",
                )
                .channel_types(vec![ChannelType::Text])
                .required(true)
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "counting_channel",
                    "Channel for counting",
                )
                .channel_types(vec![ChannelType::Text])
                .required(true)
            )
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "timezone",
                "Set timezone(UTC) for logging clarity",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "utc",
                    "Universal Time Coordinated",
                )
                .min_number_value(-12.0)
                .max_number_value(14.0)
                .required(true)
            )
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "language",
                "Set language for your logs",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "lang",
                    "Pick a language",
                )
                .add_string_choice("English", "en")
                .add_string_choice("Japanese", "jp")
                .required(true)
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "lang2",
                    "Pick a 2nd language",
                )
                .add_string_choice("English", "en")
                .add_string_choice("Japanese", "jp")
                .required(false)
            )
        )
}

pub async fn execute(ctx: Context, command: CommandInteraction, bot_data: &BotData) {
    if !check_admin(&ctx, &command).await { return; }

    if let Some(guild_id) = command.guild_id {
        let guild_id_u64 = guild_id.get();
        let mut guild_data = load_guild_data(guild_id_u64);

        if let Some(top) = command.data.options.first() {
            match top.name.as_str() {
                "info" => {
                    if !guild_data.is_setup {
                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("❗ This server hasn't been setup yet!\nPlease use `/setup channels` to setup necessary channels.")
                                .flags(InteractionResponseFlags::EPHEMERAL)
                        )).await {
                            internal_err(&ctx, &command, &e.to_string()).await;
                        }
                    } else if let (
                        Some(log_ch_id),
                        Some(count_ch_id)
                    ) = (
                        guild_data.ids.log_channel_id,
                        guild_data.ids.counting_channel_id
                    ) {
                        let utc_format = get_utc_format(&guild_data.settings.utc);
                        
                        let reply = format!(
                            "`UTC {}`\n`lang`: {}\n`lang2`: {}\n`log_channel`: <#{}>\n`counting_channel`: <#{}>",
                            utc_format,
                            guild_data.settings.lang,
                            guild_data.settings.lang2.as_deref().unwrap_or("(none)"),
                            log_ch_id,
                            count_ch_id
                        );

                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content(reply)
                                .flags(InteractionResponseFlags::EPHEMERAL)
                        )).await {
                            internal_err(&ctx, &command, &e.to_string()).await;
                        }
                    } else {
                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("❗ Missing Channel(s) in Configuration!\nPlease use `/setup channels` to setup channels again.")
                                .flags(InteractionResponseFlags::EPHEMERAL)
                        )).await {
                            internal_err(&ctx, &command, &e.to_string()).await;
                        }
                    }
                }

                "reset" => {
                    let is_default = guild_data.is_default_setup();

                    if let Some(log_ch_id) = guild_data.ids.log_channel_id {
                        let log_channel = ChannelId::new(log_ch_id);
                        while let Ok(msgs) = log_channel.messages(&ctx.http, GetMessages::new().limit(100)).await {
                            let bot_msgs: Vec<_> = msgs.into_iter().filter(|m| m.author.bot).collect();
                            if bot_msgs.is_empty() { break; }
                            if bot_msgs.len() > 1 {
                                let _ = log_channel.delete_messages(&ctx.http, &bot_msgs).await;
                            } else {
                                let _ = log_channel.delete_message(&ctx.http, bot_msgs[0].id).await;
                            }
                        }
                    }

                    if !guild_data.is_setup {
                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("❗ This server hasn't been setup yet!\nYou can use `/setup channels` to setup necessary channels.")
                                .flags(InteractionResponseFlags::EPHEMERAL)
                        )).await {
                            internal_err(&ctx, &command, &e.to_string()).await;
                        }
                    } else if !is_default {
                        guild_data = GuildData::default();
                        guild_data.is_setup = false;

                        {
                            let mut guilds = bot_data.guilds.lock().await;
                            guilds.insert(guild_id_u64, guild_data.clone());
                        }
                        save_guild_data(guild_id_u64, &guild_data);

                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("✅ Reset Done!")
                                .flags(InteractionResponseFlags::EPHEMERAL)
                        )).await {
                            internal_err(&ctx, &command, &e.to_string()).await;
                        }

                        log_info(format!("🛠 Reset Done for Guild{}", guild_id_u64).as_str());
                    } else {
                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("✅ Reset Done!")
                                .flags(InteractionResponseFlags::EPHEMERAL)
                        )).await {
                            internal_err(&ctx, &command, &e.to_string()).await;
                        }

                        log_info(format!("🛠 Reset Done for Guild{}", guild_id_u64).as_str());
                    }
                }

                "channels" => { if let CommandDataOptionValue::SubCommand(sub_options) = &top.value {
                    let log_option = sub_options
                        .iter()
                        .find(|o| o.name == "log_channel")
                        .and_then(|o| o.value.as_channel_id());

                    let count_option = sub_options
                        .iter()
                        .find(|o| o.name == "counting_channel")
                        .and_then(|o| o.value.as_channel_id());

                    if let (Some(new_log_channel), Some(new_count_channel)) = (log_option, count_option) {
                        if new_log_channel.get() == new_count_channel.get() {
                            if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("❌ Can't set `log_channel` and `counting_channel` as the same.\nPlease try again")
                                    .flags(InteractionResponseFlags::EPHEMERAL)
                            )).await {
                                internal_err(&ctx, &command, &e.to_string()).await;
                            }

                            return;
                        }

                        if let Some(log_ch_id) = guild_data.ids.log_channel_id {
                            let old_log_channel = ChannelId::new(log_ch_id);

                            while let Ok(msgs) = old_log_channel.messages(&ctx.http, GetMessages::new().limit(100)).await {
                                let bot_msgs: Vec<_> = msgs.into_iter().filter(|m| m.author.bot).collect();
                                if bot_msgs.is_empty() { break; }
                                if bot_msgs.len() > 1 {
                                    let _ = old_log_channel.delete_messages(&ctx.http, &bot_msgs).await;
                                } else {
                                    let _ = old_log_channel.delete_message(&ctx.http, bot_msgs[0].id).await;
                                }
                            }
                        }

                        while let Ok(msgs) = new_log_channel.messages(&ctx.http, GetMessages::new().limit(100)).await {
                            let bot_msgs: Vec<_> = msgs.into_iter().filter(|m| m.author.bot).collect();
                            if bot_msgs.is_empty() { break; }
                            if bot_msgs.len() > 1 {
                                let _ = new_log_channel.delete_messages(&ctx.http, &bot_msgs).await;
                            } else {
                                let _ = new_log_channel.delete_message(&ctx.http, bot_msgs[0].id).await;
                            }
                        }

                        guild_data.ids.log_channel_id = Some(new_log_channel.get());
                        guild_data.ids.counting_channel_id = Some(new_count_channel.get());
                        guild_data.is_setup = true;

                        {
                            let mut guilds = bot_data.guilds.lock().await;
                            guilds.insert(guild_id_u64, guild_data.clone());
                        }
                        save_guild_data(guild_id_u64, &guild_data);

                        let reply = format!(
                            "✅ Setup Done!\n`log_channel`: <#{}>\n`counting_channel`: <#{}>\n\nPlease ensure that this bot actually has necessary permissions for said channels set.",
                            new_log_channel.get(),
                            new_count_channel.get()
                        );

                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content(reply)
                                .flags(InteractionResponseFlags::EPHEMERAL)
                        )).await {
                            internal_err(&ctx, &command, &e.to_string()).await;
                        }

                        log_info(format!("🛠 Setup 'Channel' Done for Guild{}", guild_id_u64).as_str());

                        crate::commands::relog::execute(ctx, command, bot_data).await;
                    } else {
                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("❌ Can't set channels individually.\nPlease try again and set all necessary channels.")
                                .flags(InteractionResponseFlags::EPHEMERAL)
                        )).await {
                            internal_err(&ctx, &command, &e.to_string()).await;
                        }

                        return;
                    }
                }}

                "timezone" => { if let CommandDataOptionValue::SubCommand(sub_options) = &top.value {
                    let timezone_option = sub_options
                        .iter()
                        .find(|o| o.name == "utc")
                        .and_then(|o| o.value.as_i64());

                    if let Some(new_timezone) = timezone_option {
                        guild_data.settings.utc = new_timezone.clamp(-12, 14) as i8;

                        {
                            let mut guilds = bot_data.guilds.lock().await;
                            guilds.insert(guild_id_u64, guild_data.clone());
                        }
                        save_guild_data(guild_id_u64, &guild_data);

                        let utc_format = get_utc_format(&guild_data.settings.utc);

                        let reply = format!(
                            "✅ Setup Done!\n`UTC {}`",
                            utc_format.as_str()
                        );

                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content(reply)
                                .flags(InteractionResponseFlags::EPHEMERAL)
                        )).await {
                            internal_err(&ctx, &command, &e.to_string()).await;
                        }

                        log_info(format!("🛠 Setup 'Timezone' Done for Guild{}", guild_id_u64).as_str());
                    } else {
                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("❌ Missing `utc` variable.\nPlease try again")
                                .flags(InteractionResponseFlags::EPHEMERAL)
                        )).await {
                            internal_err(&ctx, &command, &e.to_string()).await;
                        }

                        return;
                    }
                }}

                "language" => { if let CommandDataOptionValue::SubCommand(sub_options) = &top.value {
                    let lang_option = sub_options
                        .iter()
                        .find(|o| o.name == "lang")
                        .and_then(|o| o.value.as_str());

                    let lang2_option = sub_options
                        .iter()
                        .find(|o| o.name == "lang2")
                        .and_then(|o| o.value.as_str());

                    if let Some(new_lang) = lang_option {
                        let langs = vec!["en", "jp"];

                        let valid_lang2 = lang2_option.map_or(true, |l| langs.contains(&l));

                        if !langs.contains(&new_lang) || !valid_lang2 {
                            if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("❌ Can't set languages other than available selections.\nPlease try again")
                                    .flags(InteractionResponseFlags::EPHEMERAL)
                            )).await {
                                internal_err(&ctx, &command, &e.to_string()).await;
                            }

                            return;
                        }

                        guild_data.settings.lang = new_lang.to_string();
                        guild_data.settings.lang2 = lang2_option.map(str::to_string);

                        {
                            let mut guilds = bot_data.guilds.lock().await;
                            guilds.insert(guild_id_u64, guild_data.clone());
                        }
                        save_guild_data(guild_id_u64, &guild_data);

                        let reply = format!(
                            "✅ Setup Done!\n`lang`: {}\n`lang2`: {}",
                            new_lang,
                            lang2_option.unwrap_or("(none)")
                        );

                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content(reply)
                                .flags(InteractionResponseFlags::EPHEMERAL)
                        )).await {
                            internal_err(&ctx, &command, &e.to_string()).await;
                        }
                    } else {
                        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("❌ Missing `lang` variable.\nPlease try again")
                                .flags(InteractionResponseFlags::EPHEMERAL)
                        )).await {
                            internal_err(&ctx, &command, &e.to_string()).await;
                        }

                        return;
                    }
                }}

                _ => {
                    if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("❓ Available options: `info`, `reset`, `channels`, `timezone`, `language`")
                            .flags(InteractionResponseFlags::EPHEMERAL)
                    )).await {
                        internal_err(&ctx, &command, &e.to_string()).await;
                    }
                }
            }
        }
    } else {
        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("❗ This command can only be used within a discord server!")
                .flags(InteractionResponseFlags::EPHEMERAL)
        )).await {
            internal_err(&ctx, &command, &e.to_string()).await;
        }
    }
}
