use serde::{Serialize, Deserialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GuildSettings {
    pub utc: i16,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct IDs {
    pub log_channel_id: Option<u64>,
    pub counting_channel_id: Option<u64>,
    pub log_msg_ids: Vec<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GuildData {
    pub is_setup: bool,
    pub settings: GuildSettings,
    pub ids: IDs,
    pub daily_counts: BTreeMap<String, i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AllGuildData(pub HashMap<u64, GuildData>);
