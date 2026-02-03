use crate::data::structs::GuildData;
use crate::data::{BotData, load_guild_data, save_guild_data};
use crate::utils::*;
use chrono::*;
use once_cell::sync::Lazy;
use serenity::all::*;
use std::collections::BTreeSet;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant, MissedTickBehavior, interval, sleep};
use tokio_util::sync::CancellationToken;

pub fn register() -> CreateCommand {
    CreateCommand::new("relog")
        .description("Relog bundled commands")
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "start",
            "Refresh and update all logs from the start",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "cancel",
            "Cancel on-going relog session",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommandGroup,
                "auto",
                "Relog update automation",
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "toggle",
                "Toggle auto update activity",
            )),
        )
}

#[derive(Clone)]
pub struct RelogState {
    pub lock: Arc<Mutex<()>>,
    pub cancel_token: CancellationToken,
}

static RELOG_STATES: Lazy<Mutex<HashMap<u64, RelogState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

async fn get_relog_state(guild_id: u64) -> RelogState {
    let mut map = RELOG_STATES.lock().await;

    map.entry(guild_id)
        .or_insert_with(|| RelogState {
            lock: Arc::new(Mutex::new(())),
            cancel_token: CancellationToken::new(),
        })
        .clone()
}

pub async fn execute(ctx: Context, command: CommandInteraction, bot_data: &BotData) {
    if !check_admin(&ctx, &command).await {
        return;
    }

    if let Some(guild_id) = command.guild_id {
        let guild_id_u64 = guild_id.get();
        let mut guild_data = load_guild_data(guild_id_u64);

        if !guild_data.is_setup {
            if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("❗ This server hasn't been setup yet!\nPlease use `/setup channels` to setup necessary channels.")
                    .flags(InteractionResponseFlags::EPHEMERAL)
            )).await {
                internal_err(&ctx, &command, &e.to_string()).await;
            }
            return;
        }

        if let Some(top) = command.data.options.first() {
            match top.name.as_str() {
                "start" => {
                    if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("📝 Relog underway...\n-# If you do not see the follow up message, please make sure CountLogger has `Send Messages` permission.")
                            .flags(InteractionResponseFlags::EPHEMERAL)
                    )).await {
                        internal_err(&ctx, &command, &e.to_string()).await;
                    }

                    let state = get_relog_state(guild_id_u64).await;
                    let _guard = state.lock.lock().await;
                    let token = state.cancel_token.clone();

                    let _ = relog_start(
                        &ctx,
                        &command,
                        bot_data,
                        guild_id_u64,
                        &mut guild_data,
                        token,
                    )
                    .await;
                }

                "cancel" => {
                    let state = get_relog_state(guild_id_u64).await;
                    if state.lock.try_lock().is_ok() {
                        if let Err(e) = command
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content("❌ No on-going relog session active")
                                        .flags(InteractionResponseFlags::EPHEMERAL),
                                ),
                            )
                            .await
                        {
                            internal_err(&ctx, &command, &e.to_string()).await;
                        }
                        return;
                    } else {
                        state.cancel_token.cancel();
                        if let Err(e) = command
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content("✅ Cancelled on-going relog session")
                                        .flags(InteractionResponseFlags::EPHEMERAL),
                                ),
                            )
                            .await
                        {
                            internal_err(&ctx, &command, &e.to_string()).await;
                        }
                    }
                }

                "auto" => {
                    if let CommandDataOptionValue::SubCommandGroup(group_options) = &top.value {
                        if let Some(toggle) = group_options.iter().find(|o| o.name == "toggle") {
                            if let CommandDataOptionValue::SubCommand(toggle_options) =
                                &toggle.value
                            {
                                if toggle_options.is_empty() {
                                    let mut auto_relog = guild_data.settings.auto_relog;
                                    auto_relog = !auto_relog;
                                    guild_data.settings.auto_relog = auto_relog;

                                    if let Err(e) = command
                                        .create_response(
                                            &ctx.http,
                                            CreateInteractionResponse::Message(
                                                CreateInteractionResponseMessage::new()
                                                    .content(format!(
                                                        "✅ Toggled auto relog\n`{}` -> `{}`",
                                                        !auto_relog, auto_relog
                                                    ))
                                                    .flags(InteractionResponseFlags::EPHEMERAL),
                                            ),
                                        )
                                        .await
                                    {
                                        internal_err(&ctx, &command, &e.to_string()).await;
                                    }

                                    {
                                        let mut guilds = bot_data.guilds.lock().await;
                                        guilds.insert(guild_id_u64, guild_data.clone());
                                    }
                                    save_guild_data(guild_id_u64, &guild_data);
                                }
                            }
                        }
                    }
                }

                _ => {
                    if let Err(e) = command
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("❓ Available options: `start`, `cancel`, `auto`")
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

async fn relog_start(
    ctx: &Context,
    command: &CommandInteraction,
    bot_data: &BotData,
    guild_id_u64: u64,
    guild_data: &mut GuildData,
    token: CancellationToken,
) -> Result<(), ()> {
    if let (Some(count_ch_id), Some(log_ch_id)) = (
        guild_data.ids.counting_channel_id,
        guild_data.ids.log_channel_id,
    ) {
        guild_data.ids.last_scanned_msg_id = None;

        let log_channel = ChannelId::new(log_ch_id);

        let progress_msg = log_channel
            .send_message(
                &ctx.http,
                CreateMessage::new().content("🔄 Relog in progress... this might take a while!"),
            )
            .await
            .unwrap_or_default();

        match get_lastmsg_day_map(
            &ctx,
            &progress_msg.id,
            log_channel,
            ChannelId::new(count_ch_id),
            &guild_data.settings.utc,
            token,
        )
        .await
        {
            Ok((daily_counts, last_message_id)) => {
                let years: BTreeSet<String> = daily_counts
                    .keys()
                    .map(|key| key.split('-').next().unwrap().to_string())
                    .collect();

                if let Some(new_last) = last_message_id {
                    guild_data.ids.last_scanned_msg_id = Some(new_last.get());
                }

                let mut new_log_msg_map: BTreeMap<i32, BTreeMap<i64, u64>> = BTreeMap::new();
                let mut last_year_latest_count = 0i64;

                for year in years {
                    let year_i: i32 = year.parse().unwrap_or(0);
                    let year_counts: BTreeMap<String, i64> = daily_counts
                        .iter()
                        .filter(|(k, _)| k.starts_with(&year))
                        .map(|(k, v)| (k.clone(), *v))
                        .collect();

                    let new_log_msgs = generate_log_messages(
                        &guild_data,
                        year_counts.clone(),
                        Some(last_year_latest_count),
                    );
                    let mut year_map: BTreeMap<i64, u64> = BTreeMap::new();

                    {
                        last_year_latest_count = *year_counts.last_key_value().unwrap().1;
                    }

                    for (part, new_log_msg) in new_log_msgs {
                        if let Some(old_id) = guild_data
                            .ids
                            .log_msg_map
                            .get(&year_i)
                            .and_then(|ym| ym.get(&part))
                        {
                            if log_channel.message(&ctx.http, *old_id).await.is_ok() {
                                let _ = log_channel
                                    .edit_message(
                                        &ctx.http,
                                        *old_id,
                                        EditMessage::new().content(new_log_msg),
                                    )
                                    .await;
                                year_map.insert(part, *old_id);
                                continue;
                            }
                        }

                        // fallback: create new
                        if let Ok(new_msg) = log_channel
                            .send_message(&ctx.http, CreateMessage::new().content(new_log_msg))
                            .await
                        {
                            year_map.insert(part, new_msg.id.get());
                        }
                    }

                    new_log_msg_map.insert(year_i, year_map);
                }

                // update state
                guild_data.daily_counts = daily_counts;
                guild_data.ids.log_msg_map = new_log_msg_map;

                log_info(&format!(
                    "🛠 Relog Done for Guild{} ({} entries)",
                    guild_id_u64,
                    guild_data.daily_counts.len()
                ));
            }
            Err(e) => {
                internal_err(&ctx, &command, &e.to_string()).await;

                let _ = log_channel
                    .edit_message(
                        &ctx.http,
                        progress_msg.id,
                        EditMessage::new().content(
                            "❗Relog Session Interrupted!\n-# This message will delete automatically in 10 seconds",
                        ),
                    )
                    .await;

                sleep(Duration::from_secs(10)).await;

                let _ = progress_msg.delete(&ctx.http).await;

                return Err(());
            }
        }

        {
            let mut guilds = bot_data.guilds.lock().await;
            guilds.insert(guild_id_u64, guild_data.clone());
        }
        save_guild_data(guild_id_u64, &guild_data);

        let _ = log_channel
            .edit_message(
                &ctx.http,
                progress_msg.id,
                EditMessage::new().content(
                    "✅ Relog Done!\n-# This message will delete automatically in 10 seconds",
                ),
            )
            .await;

        sleep(Duration::from_secs(10)).await;

        let _ = progress_msg.delete(&ctx.http).await;
    }

    Ok(())
}

pub async fn log_daily_counts(ctx: Context, bot_data: Arc<BotData>) {
    let mut interval = interval(chrono::Duration::minutes(5).to_std().unwrap());
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        let guilds = bot_data.guilds.lock().await.clone();
        for (guild_id_u64, mut guild_data) in guilds {
            let state = get_relog_state(guild_id_u64).await;
            if state.lock.try_lock().is_err() || !guild_data.settings.auto_relog {
                continue;
            }

            if let Some(count_ch_id) = guild_data.ids.counting_channel_id {
                if let Some(log_ch_id) = guild_data.ids.log_channel_id {
                    let count_channel = ChannelId::new(count_ch_id);
                    let log_channel = ChannelId::new(log_ch_id);

                    match fetch_new_daily_counts(
                        &ctx.http,
                        count_channel,
                        &guild_data.settings.utc,
                        guild_data.ids.last_scanned_msg_id.map(MessageId::new),
                    )
                    .await
                    {
                        Ok((new_counts, last_seen)) => {
                            // merge into existing counts
                            for (date, count) in new_counts {
                                guild_data
                                    .daily_counts
                                    .entry(date)
                                    .and_modify(|v| *v = (*v).max(count))
                                    .or_insert(count);
                            }
                            if let Some(new_last) = last_seen {
                                guild_data.ids.last_scanned_msg_id = Some(new_last.get());
                            }

                            // update ONLY current year logs
                            let offset =
                                FixedOffset::east_opt(guild_data.settings.utc as i32 * 3600)
                                    .unwrap();
                            let year_now = Utc::now().with_timezone(&offset).year();

                            let year_counts: BTreeMap<String, i64> = guild_data
                                .daily_counts
                                .iter()
                                .filter(|(k, _)| k.starts_with(&year_now.to_string()))
                                .map(|(k, v)| (k.clone(), *v))
                                .collect();

                            let new_log_msgs =
                                generate_log_messages(&guild_data, year_counts, None);

                            let mut year_map = guild_data
                                .ids
                                .log_msg_map
                                .remove(&year_now)
                                .unwrap_or_default();

                            for (part, new_msg) in new_log_msgs.clone() {
                                if let Some(&old_id) = year_map.get(&part) {
                                    let _ = log_channel
                                        .edit_message(
                                            &ctx.http,
                                            MessageId::new(old_id),
                                            EditMessage::new().content(new_msg),
                                        )
                                        .await;
                                } else if let Ok(new_msg) = log_channel
                                    .send_message(&ctx.http, CreateMessage::new().content(new_msg))
                                    .await
                                {
                                    year_map.insert(part, new_msg.id.get());
                                }
                            }

                            // cleanup: if year_map had leftover parts not regenerated
                            let valid_parts: Vec<i64> = new_log_msgs.keys().cloned().collect();
                            year_map.retain(|part, _| valid_parts.contains(part));

                            guild_data.ids.log_msg_map.insert(year_now, year_map);

                            {
                                let mut guilds = bot_data.guilds.lock().await;
                                guilds.insert(guild_id_u64, guild_data.clone());
                            }
                            save_guild_data(guild_id_u64, &guild_data);
                        }
                        Err(e) => {
                            log_error(&format!(
                                "Failed fetching new counts for Guild{}: {}",
                                guild_id_u64, e
                            ));
                        }
                    }
                }
            }
        }
    }
}

// Fetch ONLY messages after last_scanned_msg_id
async fn fetch_new_daily_counts(
    http: &Http,
    channel_id: ChannelId,
    utc: &i8,
    last_scanned: Option<MessageId>,
) -> serenity::Result<(BTreeMap<String, i64>, Option<MessageId>)> {
    let mut daily_counts: BTreeMap<String, i64> = BTreeMap::new();
    let mut last_seen: Option<MessageId> = None;
    let mut last_num = 0i64;

    let mut get_message = GetMessages::new().limit(100);
    if let Some(last_id) = last_scanned {
        get_message = get_message.after(last_id);
    }

    loop {
        let msgs = channel_id.messages(http, get_message.clone()).await?;
        if msgs.is_empty() {
            break;
        }

        let mut page_msgs = msgs;
        page_msgs.reverse();

        for msg in &page_msgs {
            if msg.author.bot {
                continue;
            }
            if let Ok(num) = msg.content.parse::<i64>() {
                if let Some(offset) =
                    FixedOffset::east_opt(utc.clone().clamp(-14, 12) as i32 * 3600)
                {
                    let dt: DateTime<FixedOffset> = msg.timestamp.with_timezone(&offset);
                    let key = dt.date_naive().format("%Y-%m-%d").to_string();
                    if last_num == 0 || is_valid_num(last_num, num) {
                        daily_counts
                            .entry(key)
                            .and_modify(|v| *v = (*v).max(num))
                            .or_insert(num);
                    }
                    last_num = num;
                }
            }
        }

        last_seen = page_msgs.last().map(|m| m.id);

        // break if fewer than 100 messages fetched → no more new msgs
        if page_msgs.len() < 100 {
            break;
        }
        get_message = GetMessages::new()
            .limit(100)
            .after(page_msgs.last().unwrap().id);
    }

    Ok((daily_counts, last_seen))
}

async fn get_lastmsg_day_map(
    ctx: &Context,
    progress_msg: &MessageId,
    log_channel_id: ChannelId,
    count_channel_id: ChannelId,
    utc: &i8,
    token: CancellationToken,
) -> serenity::Result<
    (BTreeMap<String, i64>, Option<MessageId>),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let mut daily_counts: BTreeMap<String, i64> = BTreeMap::new();
    let mut last_message_id: Option<MessageId> = None;
    let mut last_num = 0i64;

    let mut last_update = Instant::now();
    let unix_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let start_timestamp = format!("<t:{}:R>", unix_time);

    let mut total_count = 0i64;
    let mut total_invalid_detected = 0i64;

    loop {
        let mut get_message = GetMessages::new().limit(100);
        if let Some(last_id) = last_message_id {
            get_message = get_message.before(last_id);
        }
        let msgs = count_channel_id.messages(&ctx.http, get_message).await?;
        if msgs.is_empty() {
            break;
        }

        let mut page_msgs = msgs;
        page_msgs.reverse();

        for msg in &page_msgs {
            if token.is_cancelled() {
                return Err("CancelledToken".into());
            }

            if msg.author.bot {
                continue;
            }
            if let Ok(num) = msg.content.parse::<i64>() {
                if total_count == 0 {
                    total_count = num
                };
                if let Some(offset) =
                    FixedOffset::east_opt(utc.clone().clamp(-14, 12) as i32 * 3600)
                {
                    if last_update.elapsed() >= Duration::from_secs(10) {
                        let unix_time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let update_timestamp = format!("<t:{}:R>", unix_time);

                        let _ = log_channel_id
                            .edit_message(
                                &ctx.http,
                                progress_msg,
                                EditMessage::new().content(format!(
                                    "🔄 Relog in progress...\n📊 Read Counts Left: `{}/{}`{}\n-# Started {}\n-# Last Update {}",
                                    &num,
                                    &total_count,
                                    if total_invalid_detected > 0 {
                                        format!("\nInvalid Counts Detected: `{}`", total_invalid_detected)
                                    } else {
                                        "".to_string()
                                    },
                                    start_timestamp,
                                    update_timestamp
                                )),
                            )
                            .await;

                        last_update = Instant::now();
                    }

                    let dt: DateTime<FixedOffset> = msg.timestamp.with_timezone(&offset);
                    let key = dt.date_naive().format("%Y-%m-%d").to_string();
                    if last_num != 0 && !is_valid_num(last_num, num) {
                        total_invalid_detected += 1;
                    } else {
                        daily_counts
                            .entry(key)
                            .and_modify(|v| *v = (*v).max(num))
                            .or_insert(num);
                    }
                    last_num = num;
                }
            }
        }

        // guard against infinite loop
        let new_last = page_msgs.first().map(|m| m.id);
        if new_last == last_message_id {
            break;
        }
        last_message_id = new_last;
    }

    Ok((daily_counts, last_message_id))
}

fn generate_log_messages(
    guild_data: &GuildData,
    counts: BTreeMap<String, i64>,
    offset_count: Option<i64>,
) -> BTreeMap<i64, String> {
    let lang1 = guild_data.settings.lang.as_str();
    let lang2 = guild_data.settings.lang2.as_deref();
    let utc = &guild_data.settings.utc;

    let mut messages: BTreeMap<i64, String> = BTreeMap::default();
    let mut msg_lines: Vec<String> = Vec::new();
    let mut line_count = 0usize;
    let mut prev_count = match offset_count {
        Some(count) => count,
        None => 0i64,
    };
    let mut part = 1i64;

    if counts.is_empty() {
        return messages;
    }

    let last_date = counts.keys().last().cloned();

    for (date, count) in counts {
        let parts: Vec<&str> = date.split("-").collect();
        let (y, m, d) = match &parts[..] {
            [y, m, d] => (*y, *m, *d),
            _ => {
                log_error("❗ Wrong date format!");
                continue;
            }
        };

        let increment = count - prev_count;
        prev_count = count;

        let line = format!("`{m}-{d}` : **{count}** (+{increment})");
        msg_lines.push(line);
        line_count += 1;

        if line_count % 5 == 0 {
            msg_lines.push(format!("-# -{line_count}-"));
        }

        let is_last = last_date.as_ref().map(|s| s == &date).unwrap_or(false);

        if is_last && line_count % 5 != 0 {
            msg_lines.push(format!("-# -{line_count}-"));
        }

        /* anymore than 40 lines (safe limit)
        will result in unmarked-down message
        Don't ask me why discord is like this*/
        if line_count >= 40 || is_last {
            // "## **📊 `Year {}` Count Log:**\n`date : sum  (5 min update)`\n"
            let header = format!(
                "## **📊 `{} {} ({})` {}**\n`{} (UTC {}) : {}  ({})`\n",
                get_word("Year", lang1, lang2, CharaCase::Normal),
                y.to_string(),
                part.to_string(),
                get_word("Count Log", lang1, None, CharaCase::Normal),
                get_word("Date", lang1, lang2, CharaCase::Normal),
                get_utc_format(utc),
                get_word("Sum", lang1, lang2, CharaCase::Normal),
                get_word("5 minutes change", lang1, None, CharaCase::Normal),
            );
            messages.insert(part, format!("{}{}", header, msg_lines.join("\n")));
            msg_lines.clear();
            line_count = 0;
            part += 1;
        }
    }

    messages
}

fn is_valid_num(pre: i64, post: i64) -> bool {
    pre + 1 == post
}
