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
> * Python 3.9+ `(current: 3.13.5)`
> * discord.py 2.3.2+ `(current: 2.5.2)`
> * python-dotenv 1.0.0+ `(current: 1.1.0)`

* Clone this repo:
    `git clone https://github.com/BB-69/CountLogger.git`

* Build local files & install dependencies:
    `python build.py` (linux) || `py build.py` (windows)
    * Build `.env` file with specified `DISCORD_TOKEN` `TEST_GUILD_ID` `BOT_OWNER_ID`
    * Build empty file `data.json` with `{}`
    * Build empty file `config.json` with `{}`
    * Optional: Install all dependencies via `requirements.txt`

* Run the bot:
    `python main.py` (linux) || `py main.py` (windows)

## ⚙️ Configuration
* In `data.json`:
```
{
  "<guild_id>": {
      "log_channel_id": <log_channel_id>,
      "counting_channel_id": <counting_channel_id>
  },
  ...
}
```
* In `config.json`:
```
{
  "<guild_id>:<YYYY/MM/DD>": <count>,
  ...
}
```

## 💬 Commands
> Haven't settle commands type yet...

* Text commands:
    * `!ctl help` : full guide about this bot
    * `!ctl setup` : view your current channel set up
    * `!ctl setup <your_log_channel> <your_counting_channel>` : set each specified channel as current
    * `!ctl relog` : recalculate and update all count logs in `<your_log_channel>`

* Slash commands:
    * `/helpcmd` : full guide about this bot
    * `/setupinfo` : view your current channel set up
    * `/setup <your_log_channel> <your_counting_channel>` : set each specified channel as current
    * `/relog` : recalculate and update all count logs in `<your_log_channel>`

## 🔒 OAuth2 Bot Permissions
* Send Messages ✅
* Manage Messages ✅
* Read Message history ✅

## 📥 Contribution
Idk just do pr or something kek, i'll check
