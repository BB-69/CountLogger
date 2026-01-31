use dotenv::dotenv;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serenity::builder::*;
use serenity::model::application::*;
use serenity::prelude::*;
use std::collections::HashMap;
use std::env;
use std::fs;

pub fn log_info(msg: &str) {
    println!("[INFO]: {}", msg)
}
pub fn log_warn(msg: &str) {
    println!("[WARN]: {}", msg)
}
pub fn log_error(msg: &str) {
    eprintln!("[ERROR]: {}", msg)
}

pub async fn internal_err(ctx: &Context, command: &CommandInteraction, err: &str) {
    let msg = format!("INTERNAL_ERROR: `{}`", err);

    let _ = command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(&msg)
                    .flags(InteractionResponseFlags::EPHEMERAL),
            ),
        )
        .await;

    log_error(&msg);
}

pub async fn check_admin(ctx: &Context, command: &CommandInteraction) -> bool {
    let member = command.member.as_ref().unwrap();

    if member.permissions.unwrap_or_default().administrator() {
        return true;
    }

    if dotenv().is_ok() {
        let owner_id = env::var("BOT_OWNER_ID").unwrap_or_default();
        if member.user.id.get().to_string() == owner_id {
            return true;
        };
    }

    if let Err(e) = command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("⛔ You need admin privileges to use this command!")
                    .flags(InteractionResponseFlags::EPHEMERAL),
            ),
        )
        .await
    {
        internal_err(&ctx, &command, &e.to_string()).await;
    }

    false
}

#[derive(Deserialize)]
struct Dictionary(HashMap<String, HashMap<String, String>>);

static DICTIONARY: Lazy<HashMap<String, HashMap<String, String>>> = Lazy::new(|| {
    let data = fs::read_to_string("src/dictionary.json").expect("Failed to read 'dictionary.json'");
    let dict: Dictionary = serde_json::from_str(&data).expect("Failed to parse 'dictionary.json'");
    dict.0
});
pub enum CharaCase {
    Upper,
    Normal,
    Lower,
}
fn apply_case(word: &str, case: &CharaCase) -> String {
    match case {
        CharaCase::Upper => word.to_uppercase(),
        CharaCase::Lower => word.to_lowercase(),
        CharaCase::Normal => word.to_string(),
    }
}

fn lookup<'a>(dict: &'a HashMap<String, String>, lang: &str) -> Option<&'a str> {
    dict.get(lang)
        .map(|s| s.as_str())
        .or_else(|| dict.get("en").map(|s| s.as_str()))
}

pub fn get_word(word: &str, lang1: &str, lang2: Option<&str>, case: CharaCase) -> String {
    let word_dict = match DICTIONARY.get(&word.to_uppercase()) {
        Some(d) => d,
        None => return "<null>".to_string(),
    };

    let mut result = String::new();

    // lang1
    let primary = lookup(word_dict, lang1).unwrap_or("<null>");
    result.push_str(&apply_case(primary, &case));

    // lang2
    if let Some(lang2) = lang2 {
        let secondary = lookup(word_dict, lang2).unwrap_or("<null>");
        result.push('/');
        result.push_str(&apply_case(secondary, &case));
    }

    result
}

pub fn get_utc_format(utc: &i8) -> String {
    if *utc < 0 {
        "".to_owned() + &utc.to_string()
    } else {
        "+".to_owned() + &utc.to_string()
    }
}
