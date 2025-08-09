use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GuildSettings {
    pub test: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct IDs {
    pub channel_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GuildData {
    pub is_setup: bool,
    pub settings: GuildSettings,
    pub ids: IDs,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AllGuildData(pub HashMap<u64, GuildData>);
