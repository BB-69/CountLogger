use crate::data::{BotData, load_guild_data};
use crate::utils::{check_admin, internal_err};
use serenity::all::*;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{Duration, Instant, sleep};

pub fn register() -> CreateCommand {
    CreateCommand::new("message")
        .description("Message bundled commands related to CountLogger")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "purge",
                "Purge all messages from CountLogger in log_channel",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "total_message",
                    "Customize number of messages to purge",
                )
                .max_int_value(500)
                .required(false),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Boolean,
                    "include_users",
                    "Also purge other user messages",
                )
                .required(false),
            ),
        )
}

pub async fn execute(ctx: Context, command: CommandInteraction, bot_data: &BotData) {
    if !check_admin(&ctx, &command).await {
        return;
    }

    if let Some(guild_id) = command.guild_id {
        let guild_id_u64 = guild_id.get();
        let guild_data = load_guild_data(guild_id_u64);

        if let Some(top) = command.data.options.first() {
            match top.name.as_str() {
                "purge" => {
                    if let CommandDataOptionValue::SubCommand(sub_options) = &top.value {
                        if let Some(log_ch_id) = guild_data.ids.log_channel_id {
                            let _ = command.defer(&ctx.http).await;

                            let log_channel = ChannelId::new(log_ch_id);

                            let progress_msg = log_channel
                                .send_message(
                                    &ctx.http,
                                    CreateMessage::new().content(
                                        "🔄 Deletion in progress... this might take a while!",
                                    ),
                                )
                                .await
                                .unwrap_or_default();

                            let total_messages = sub_options
                                .iter()
                                .find(|o| o.name == "total_messages")
                                .and_then(|o| o.value.as_i64());
                            let include_users = sub_options
                                .iter()
                                .find(|o| o.name == "include_users")
                                .and_then(|o| o.value.as_bool());

                            if let Some(total) = total_messages {
                                if let Err(e) = delete_bot_messages(
                                    &ctx,
                                    &progress_msg.id,
                                    log_channel,
                                    Some(total),
                                    include_users,
                                )
                                .await
                                {
                                    internal_err(&ctx, &command, &e.to_string()).await;
                                }
                                return;
                            } else {
                                if let Err(e) = delete_bot_messages(
                                    &ctx,
                                    &progress_msg.id,
                                    log_channel,
                                    None,
                                    include_users,
                                )
                                .await
                                {
                                    internal_err(&ctx, &command, &e.to_string()).await;
                                }
                            }

                            let _ = log_channel
                                .edit_message(
                                    &ctx.http,
                                    progress_msg.id,
                                    EditMessage::new().content("✅ Deletion Done!\n-# This message will delete automatically in 10 seconds"),
                                )
                                .await;

                            sleep(Duration::from_secs(10)).await;

                            let _ = progress_msg.delete(&ctx.http).await;
                        }
                    }
                }

                _ => {
                    if let Err(e) = command
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("❓ Available options: `purge`")
                                    .flags(InteractionResponseFlags::EPHEMERAL),
                            ),
                        )
                        .await
                    {
                        internal_err(&ctx, &command, &e.to_string()).await;
                    }
                }
            }
        }
    } else {
        if let Err(e) = command
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("❗ This command can only be used within a discord server!")
                        .flags(InteractionResponseFlags::EPHEMERAL),
                ),
            )
            .await
        {
            internal_err(&ctx, &command, &e.to_string()).await;
        }
    }
}

async fn delete_bot_messages(
    ctx: &Context,
    progress_msg: &MessageId,
    channel_id: ChannelId,
    max_delete: Option<i64>,
    include_users: Option<bool>,
) -> serenity::Result<()> {
    let progress_msg_id = progress_msg.get();

    let mut last_message_id: Option<MessageId> = None;
    let mut deleted = 0i64;

    let mut last_update = Instant::now();
    let unix_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let discord_timestamp = format!("<t:{}:R>", unix_time);

    'outer: loop {
        let mut get_messages = GetMessages::new().limit(100);
        if let Some(last_id) = last_message_id {
            get_messages = get_messages.before(last_id);
        }

        let messages = channel_id.messages(&ctx.http, get_messages).await?;
        if messages.is_empty() {
            break;
        }

        for msg in &messages {
            if msg.id == progress_msg_id {
                continue;
            }

            if let Some(max) = max_delete {
                if deleted >= max {
                    break 'outer;
                }
            }

            if include_users.unwrap_or(false)
                || msg.author.id == ctx.http.get_current_user().await?.id
            {
                if msg.delete(&ctx.http).await.is_ok() {
                    if last_update.elapsed() >= Duration::from_secs(5) {
                        let _ = channel_id
                            .edit_message(
                                &ctx.http,
                                progress_msg,
                                EditMessage::new().content(if let Some(max) = max_delete { format!(
                                    "🔄 Deleting in progress...\n🚮 Deleted: `{}/{}`\n-# Started {}",
                                    &deleted, max, discord_timestamp
                                )} else {format!(
                                    "🔄 Deleting in progress...\n🚮 Deleted: `{}`\n-# Started {}",
                                    &deleted, discord_timestamp
                                )}),
                            )
                            .await;

                        last_update = Instant::now();
                    }

                    deleted += 1;
                }
            }
        }

        let new_last = messages.last().map(|m| m.id);
        if new_last == last_message_id {
            break;
        }
        last_message_id = new_last;
    }

    Ok(())
}
