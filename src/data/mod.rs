pub mod structs;

use chrono::{DateTime, Utc};
use serde_json;
use serenity::prelude::TypeMapKey;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use structs::*;

pub struct BotData {
    pub pool: Arc<Pool<Postgres>>,
    pub(crate) start_time: DateTime<Utc>,
}
impl BotData {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self {
            pool,
            start_time: Utc::now(),
        }
    }
}

pub struct BotDataKey;
impl TypeMapKey for BotDataKey {
    type Value = Arc<BotData>;
}

// pub fn save_all_data(all_data: &AllGuildData) {
//     let json = serde_json::to_string_pretty(&all_data).unwrap();
//     std::fs::create_dir_all("src/data").unwrap();
//     std::fs::write("src/data/data.json", json).unwrap();
// }

// pub fn load_all_data() -> AllGuildData {
//     std::fs::read_to_string("src/data/data.json")
//         .ok()
//         .and_then(|s| serde_json::from_str(&s).ok())
//         .unwrap_or_default()
// }
pub async fn load_all_data(pool: &sqlx::PgPool) -> Result<AllGuildData, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        select
            guild_id,
            is_setup,
            utc, lang, lang2, auto_relog,
            log_channel_id, counting_channel_id, log_msg_map,
            last_scanned_msg_id, log_helper_msg_id,
            daily_counts
        from public.guilds
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut map = std::collections::HashMap::new();

    for r in rows {
        map.insert(
            r.guild_id as u64,
            GuildData {
                is_setup: r.is_setup,
                settings: GuildSettings {
                    utc: r.utc as i8,
                    lang: r.lang,
                    lang2: r.lang2,
                    auto_relog: r.auto_relog,
                },
                ids: IDs {
                    log_channel_id: r.log_channel_id.map(|v| v as u64),
                    counting_channel_id: r.counting_channel_id.map(|v| v as u64),
                    log_msg_map: serde_json::from_value(r.log_msg_map).unwrap(),
                    last_scanned_msg_id: r.last_scanned_msg_id.map(|v| v as u64),
                    log_helper_msg_id: r.log_helper_msg_id.map(|v| v as u64),
                },
                daily_counts: serde_json::from_value(r.daily_counts).unwrap(),
            },
        );
    }

    Ok(AllGuildData(map))
}

// pub fn save_guild_data(guild_id: u64, data: &GuildData) {
//     let mut all_data = load_all_data();
//     all_data.0.insert(guild_id, data.clone());
//     save_all_data(&all_data);
// }
pub async fn save_guild_data(
    pool: &sqlx::PgPool,
    guild_id: u64,
    data: &GuildData,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        insert into public.guilds (
            guild_id, is_setup,
            utc, lang, lang2, auto_relog,
            log_channel_id, counting_channel_id, log_msg_map,
            last_scanned_msg_id, log_helper_msg_id,
            daily_counts
        )
        values (
            $1, $2,
            $3, $4, $5, $6,
            $7, $8, $9,
            $10, $11,
            $12
        )
        on conflict (guild_id)
        do update set
            is_setup = excluded.is_setup,
            utc = excluded.utc,
            lang = excluded.lang,
            lang2 = excluded.lang2,
            auto_relog = excluded.auto_relog,
            log_channel_id = excluded.log_channel_id,
            counting_channel_id = excluded.counting_channel_id,
            log_msg_map = excluded.log_msg_map,
            last_scanned_msg_id = excluded.last_scanned_msg_id,
            log_helper_msg_id = excluded.log_helper_msg_id,
            daily_counts = excluded.daily_counts,
            updated_at = now()
        "#,
        guild_id as i64,
        data.is_setup,
        data.settings.utc as i16,
        data.settings.lang,
        data.settings.lang2,
        data.settings.auto_relog,
        data.ids.log_channel_id.map(|v| v as i64),
        data.ids.counting_channel_id.map(|v| v as i64),
        serde_json::to_value(&data.ids.log_msg_map).unwrap(),
        data.ids.last_scanned_msg_id.map(|v| v as i64),
        data.ids.log_helper_msg_id.map(|v| v as i64),
        serde_json::to_value(&data.daily_counts).unwrap(),
    )
    .execute(pool)
    .await?;

    Ok(())
}

// pub fn load_guild_data(guild_id: u64) -> GuildData {
//     // Not 'missing data' proof
//     // load_all_data().0.get(&guild_id).cloned()

//     load_or_create_guild_data(guild_id)
// }
pub async fn load_guild_data(pool: &sqlx::PgPool, guild_id: u64) -> Result<GuildData, sqlx::Error> {
    if let Some(data) = try_load_guild_data(pool, guild_id).await? {
        return Ok(data);
    }

    // create default if missing
    let default = GuildData::default();
    save_guild_data(pool, guild_id, &default).await?;

    crate::utils::log_info(&format!("📥 Created missing guild data for '{}'", guild_id));

    Ok(default)
}

// fn load_or_create_guild_data(guild_id: u64) -> GuildData {
//     let mut all_data = load_all_data();

//     if let Some(data) = all_data.0.get(&guild_id) {
//         return data.clone();
//     }

//     let default_data = GuildData::default();
//     all_data.0.insert(guild_id, default_data.clone());
//     save_all_data(&all_data);

//     crate::utils::log_info(&format!("📥 Created missing guild data for '{}'", guild_id));

//     default_data
// }
async fn try_load_guild_data(
    pool: &sqlx::PgPool,
    guild_id: u64,
) -> Result<Option<GuildData>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        select
            is_setup,
            utc, lang, lang2, auto_relog,
            log_channel_id, counting_channel_id, log_msg_map,
            last_scanned_msg_id, log_helper_msg_id,
            daily_counts
        from guilds
        where guild_id = $1
        "#,
        guild_id as i64
    )
    .fetch_optional(pool)
    .await?;

    let Some(r) = row else {
        return Ok(None);
    };

    Ok(Some(GuildData {
        is_setup: r.is_setup,
        settings: GuildSettings {
            utc: r.utc as i8,
            lang: r.lang,
            lang2: r.lang2,
            auto_relog: r.auto_relog,
        },
        ids: IDs {
            log_channel_id: r.log_channel_id.map(|v| v as u64),
            counting_channel_id: r.counting_channel_id.map(|v| v as u64),
            log_msg_map: serde_json::from_value(r.log_msg_map).unwrap(),
            last_scanned_msg_id: r.last_scanned_msg_id.map(|v| v as u64),
            log_helper_msg_id: r.log_helper_msg_id.map(|v| v as u64),
        },
        daily_counts: serde_json::from_value(r.daily_counts).unwrap(),
    }))
}
