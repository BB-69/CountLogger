# 🧮 CountLogger
A discord bot that can log progress of a counting channel in your guild!

<img src="https://github.com/user-attachments/assets/ee97a8b5-efb2-4f5c-806e-d02a4a4ab672" width="200">

## 🪀 Features

<img src="https://github.com/user-attachments/assets/c12a75c2-de35-4b5e-a216-e8b512b22022" height="200">

* Tracks and logs daily total from a counting channel.
* Sends logs to a specific log channel & update it every certain interval `(current: 5 minutes)`.
* Customizable channel IDs via config.

## 🛠 Development

> **Dependencies:**
> _coming soon..._

* Clone this repo:
    `git clone https://github.com/BB-69/CountLogger.git`

* Build local files & install dependencies:
    `cargo run --bin fileBuild`
    * Build `.env` file with specified `DISCORD_TOKEN` `TEST_GUILD_ID` `BOT_OWNER_ID` `DATABASE_URL`

* Run the bot (dev mode):
    `cargo run --bin CountLogger`

* Build releases:
    `cargo build --release`

* Run release build:
    `./target/release/CountLogger`

## ⚙️ DB Properties

```
pub guild_id: i64,
pub is_setup: bool,

// settings
pub utc: i16,
pub lang: String,
pub lang2: Option<String>,
pub auto_relog: bool,

// ids
pub log_channel_id: Option<i64>,
pub counting_channel_id: Option<i64>,
pub log_msg_map: Value,
pub last_scanned_msg_id: Option<i64>,
pub log_helper_msg_id: Option<i64>,

// maps
pub daily_counts: Value,
```

## 💬 Commands

* `/help` : Full guide about this bot
* `/ping` : 🏓Pong! Shows basic stats

> (Admin only)

* `/setup info` : Show info of current setup
* `/setup reset` : Reset entire current setup
* `/setup channels` `[log_channel]` `[counting_channel]` : Set each specified channel as current
* `/setup timezone` `[utc]` : Set timezone for logging clarity
* `/setup language` `[lang]` `[lang2: OPTIONAL]` : Set language for logging clarity
* `/relog start` : Fetch new and update all logs from the start
* `/relog formatonly`: Refresh and update only format for logs
* `/relog end` : Cancel on-going relog session
* `/relog auto toggle` : Toggle auto update logging activity
* `/message purge` `[total_messages: OPTIONAL]` `[include_users: OPTIONAL]` : Delete all (or specified amount) of this bot's (or also other users) message from log_channel

## 📝 FORMAT

```
## 📊 Count Log
## `Year <YYYY> (<part>)`
`Date (<UTC>) : Sum`
`(5 min update)`
`<MM>-<DD>` <total_count> (+<count>)
```
> Example
>
> ### 📊 Count Log
> ### `Year 2026 (1)`
> `Date (UTC +9) : Sum`
> 
> `(5 min update)`
> 
> `01-01` 67 (+67)
> 
> `01-02` 670 (+633)
> 
> `01-03` 690 (+20)
> 
> `01-05` 911 (+221)
> 
> `02-05` 69420 (+68509)
> 
> -5-

## **-RECOMMENDED-**

- Please use this bot alongside actual counting checking bot like `Countr` or others, as this bot only purpose is to log counting history.
- Setup `[log_channel]` on an empty channel. It is dangerous to set this on a channel with message history.
- Do `/relog formatonly` (or `relog` if just setup new) to refresh and see changes everytime after done `/setup` configurations.

## 🔒 OAuth2 Bot Permissions

* Send Messages ✅
* Manage Messages ✅
* Read Message history ✅

## 📥 Contribution

Idk just do pr or something kek, i'll check
