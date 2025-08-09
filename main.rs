use serenity::all::*;
use crate::prelude::TypeMapKey;
use dotenv::dotenv;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};

fn log(msg: &str) {
    println!("[CountLogger]: {}", msg);
}

// ----- Persistence helpers -----
fn save_all_data(all_data: &AllGuildData) {
    let json = serde_json::to_string_pretty(&all_data).unwrap();
    std::fs::create_dir_all("data").unwrap();
    std::fs::write("data/data.json", json).unwrap();
}

fn load_all_data() -> AllGuildData {
    std::fs::read_to_string("data/data.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| AllGuildData(HashMap::new()))
}

fn save_guild_data(guild_id: u64, data: &GuildData) {
    let mut all_data = load_all_data();
    all_data.0.insert(guild_id, data.clone());
    save_all_data(&all_data);
}

fn load_guild_data(guild_id: u64) -> Option<GuildData> {
    let all_data = load_all_data();
    all_data.0.get(&guild_id).cloned()
}

// ----- Data Structures -----
impl Default for GuildSettings {
    fn default() -> Self {
        Self { test: 1 }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuildSettings {
    pub test: i32,
}

impl Default for IDs {
    fn default() -> Self {
        Self {
            log_channel_id: None,
            counting_channel_id: None,
            log_msg_ids: Vec::<u64>::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IDs {
    pub log_channel_id: Option<u64>,
    pub counting_channel_id: Option<u64>,
    pub log_msg_ids: Vec<u64>,
}

impl Default for GuildData {
    fn default() -> Self {
        Self {
            is_setup: false,
            settings: GuildSettings::default(),
            ids: IDs::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuildData {
    pub is_setup: bool,
    pub settings: GuildSettings,
    pub ids: IDs,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AllGuildData(pub HashMap<u64, GuildData>);

pub struct BotDataKey;

impl TypeMapKey for BotDataKey {
    type Value = Arc<BotData>;
}

pub struct BotData {
    pub guilds: Arc<Mutex<HashMap<u64, GuildData>>>,
}

// ----- Event Handler -----
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        log(&format!("✅ Bot is online as {}", ready.user.name));
    }
}

// ----- Main Entrypoint -----
#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Missing token in .env");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let bot_data = Arc::new(BotData {
        guilds: Arc::new(Mutex::new(HashMap::new())),
    });

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    {
        client.data.write().await.insert::<BotDataKey>(bot_data);
        log("Startup complete");
    }

    if let Err(e) = client.start().await {
        eprintln!("❌ Client error: {:?}", e);
    }
}
