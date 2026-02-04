use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuildSettings {
    pub utc: i8,
    pub lang: String,
    pub lang2: Option<String>,
    pub auto_relog: bool,
}
impl Default for GuildSettings {
    fn default() -> Self {
        Self {
            utc: 0,
            lang: "en".to_string(),
            lang2: None,
            auto_relog: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct IDs {
    pub log_channel_id: Option<u64>,
    pub counting_channel_id: Option<u64>,
    pub log_msg_map: BTreeMap<i32, BTreeMap<i64, u64>>, // year -> { part -> message_id }
    pub last_scanned_msg_id: Option<u64>,
    pub log_helper_msg_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GuildData {
    pub is_setup: bool,
    pub settings: GuildSettings,
    pub ids: IDs,
    pub daily_counts: BTreeMap<String, i64>,
}

impl GuildData {
    pub fn is_default_setup(&self) -> bool {
        self.settings.utc == i8::default()
            && self.ids.log_channel_id == Option::default()
            && self.ids.counting_channel_id == Option::default()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AllGuildData(pub HashMap<u64, GuildData>);
