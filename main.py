import discord
from discord import app_commands
import asyncio
from discord.ext import commands, tasks
import os
import json
from datetime import datetime, timezone, timedelta
from dotenv import load_dotenv

load_dotenv()

TOKEN = os.getenv("DISCORD_TOKEN")
test_guild = discord.Object(int(os.getenv("TEST_GUILD_ID")))
"""
LOG_CHANNEL_ID = int(os.getenv("LOG_CHANNEL_ID"))
COUNTING_CHANNEL_ID = int(os.getenv("COUNTING_CHANNEL_ID"))
"""

intents = discord.Intents.default()
intents.message_content = True
intents.messages = True
intents.guilds = True

bot = commands.Bot(command_prefix="!ctl ", intents=intents, help_command=None)

UTC_OFFSET = 9
tz = timezone(timedelta(hours=UTC_OFFSET))

data_file = "data.json"
if os.path.exists(data_file):
    with open(data_file, "r") as f:
        daily_counts = json.load(f)
else:
    daily_counts = {}
def save_data():
    with open(data_file, "w") as f:
        json.dump(daily_counts, f, indent=4)

config_file = "config.json"
def load_config():
    if os.path.exists(config_file):
        with open(config_file, "r") as f:
            return json.load(f)
    return {}
def save_config(cfg):
    with open(config_file, "w") as f:
        json.dump(cfg, f, indent=4)

config = load_config()

def is_number(msg):
    try:
        int(msg)
        return True
    except:
        return False

def log(msg):
    print(f"[CountLogger]: {msg}")

BOT_OWNER_ID = os.getenv("BOT_OWNER_ID")

def is_admin_or_owner():
    # Used when put @is_admin_or_owner() above the command function
    """
    async def predicate(ctx):
        return (
            ctx.author.id == BOT_OWNER_ID or
            is_admin_or_owner()
        )
    return commands.check(predicate)
    """

    return (
        ctx.author.id == BOT_OWNER_ID or
        is_admin_or_owner()
    )


@bot.event
async def on_ready():
    log(f"Now online as {bot.user}!")
    await bot.change_presence(
        status=discord.Status.online,
        activity=discord.Game(name="In Counting Channel With You! 💙")
    )

    await asyncio.sleep(1)

    synced = await bot.tree.sync()
    log(f"✅ Globally synced commands: {[cmd.name for cmd in synced]}")

    for guild_id in config.keys():
        await do_relog_for_guild(guild_id)
    log("✅ Relog complete for all registered guild!")
    log_daily_counts.start()

@bot.event
async def on_message(message):
    if message.author.bot or not message.guild:
        return

    guild_id = str(message.guild.id)
    guild_cfg = config.get(guild_id)
    if not guild_cfg:
        return  # not configured yet

    if message.channel.id == guild_cfg["counting_channel_id"] and is_number(message.content):
        today = datetime.now(tz).strftime("%Y/%m/%d")
        key = f"{guild_id}:{today}"
        num = int(message.content)
        if key not in daily_counts or num > daily_counts[key]:
            daily_counts[key] = num
        save_data()

    await bot.process_commands(message)

async def do_relog_for_guild(guild_id):
    if guild_id not in config:
        log(f"Guild {guild_id} not configured yet, skipping relog")
        return
    
    cfg = config[guild_id]
    counting_channel = await bot.fetch_channel(cfg["counting_channel_id"])
    if not counting_channel:
        log(f"Counting channel not found for guild {guild_id}")
        return

    # Clear previous counts
    keys_to_remove = [k for k in daily_counts if k.startswith(f"{guild_id}:")]
    for k in keys_to_remove:
        del daily_counts[k]

    # Revert to incrementing for each message
    async for message in counting_channel.history(limit=None, oldest_first=True):
        if message.author.bot:
            continue
        if message.content.isdigit():
            msg_date = message.created_at.astimezone(tz)
            day_str = msg_date.strftime("%Y/%m/%d")
            key = f"{guild_id}:{day_str}"
            num = int(message.content)
            if key not in daily_counts or num > daily_counts[key]:
                daily_counts[key] = num

    save_data()

    # Logging same as before
    years = sorted({int(k.split(":")[1].split("/")[0]) for k in daily_counts if k.startswith(f"{guild_id}:")})
    log_channel = await bot.fetch_channel(cfg["log_channel_id"])
    if not log_channel:
        log(f"Log channel not found for guild {guild_id}")
        return

    recent_bot_msgs = [msg async for msg in log_channel.history(limit=100) if msg.author == bot.user]

    for year in years:
        prefix = f"{guild_id}:{year}"
        year_counts = {
            k.split(":")[1]: count
            for k, count in daily_counts.items()
            if k.startswith(prefix)
        }

        log_msgs = generate_log_message(year, year_counts)

        for i, msg in enumerate(log_msgs):
            found = False
            for existing in recent_bot_msgs:
                if f"**📊 Year `{year} ({i+1})` Count Log:**" in existing.content:
                    await existing.edit(content=msg)
                    found = True
                    break
            if not found:
                await log_channel.send(msg)

    log(f"Relog complete for guild {guild_id}")


@tasks.loop(minutes=5)
async def log_daily_counts():
    for guild_id, cfg in config.items():
        log_channel = await bot.fetch_channel(cfg["log_channel_id"])
        if not log_channel:
            continue

        recent_bot_msgs = [msg async for msg in log_channel.history(limit=100) if msg.author == bot.user]

        year_now = datetime.now(tz).year

        # Only get the counts for this guild + year
        prefix = f"{guild_id}:{year_now}"
        counts = {
            date_key.split(":")[1]: count
            for date_key, count in daily_counts.items()
            if date_key.startswith(prefix)
        }

        log_msgs = generate_log_message(year_now, counts)

        for i, msg in enumerate(log_msgs):
            found = False
            for existing in recent_bot_msgs:
                if f"**📊 Year `{year_now} ({i+1})` Count Log:**" in existing.content:
                    await existing.edit(content=msg)
                    found = True
                    break
            if not found:
                await log_channel.send(msg)



@bot.command(name="help")
async def help_command(ctx):
    if not is_admin_or_owner():
        # await ctx.send("🚫 You need admin perms to run this!")
        return

    help_msg = """
A bot that can log progress of a counting channel in your guild!

## **-USAGE-**
Let it run and it will automatically update your logs every `5 minutes`

## **-COMMAND-**
`!c help` : full guide about this bot
`!c setup` : view your current channel set up
`!c setup <your_log_channel> <your_counting_channel>` : set each specified channel as current
`!c relog` : recalculate and update all count logs in `<your_log_channel>`

## **-FORMAT-**
**📊 Year `<year>` Count Log:**
`日にち/date : 合計/sum  (5minutes change)`
`YYYY/MM/DD` : `<total_count> (<count>)`

### **-DISCLAIMER-**
This will currently detect **all** the number in `<your_counting_channel>` regardless of order.
We recommend using another bot with proper counting rules checking for now.
"""
    await ctx.send(help_msg)

@bot.command(name="setup")
async def setup(ctx, log_channel: discord.TextChannel = None, counting_channel: discord.TextChannel = None):
    if not is_admin_or_owner():
        # await ctx.send("🚫 You need admin perms to run this!")
        return

    """
    if not log_channel or not counting_channel:
        await ctx.send("⚠️ Invalid format! Try: `!c setup #your_log_channel #your_counting_channel`")
        return
    """
    guild_id = str(ctx.guild.id)

    if not log_channel or not counting_channel:
        guild_cfg = config.get(guild_id)

        if guild_cfg is None:
            await ctx.send("❗ This server hasn't been set up yet! Use: `!c setup #your_log_channel #your_counting_channel`")
        else:
            await ctx.send(f"📤 Log Channel: <#{guild_cfg.get('log_channel_id')}>, Counting Channel: <#{guild_cfg.get('counting_channel_id')}>")
        return

    config[guild_id] = {
        "log_channel_id": log_channel.id,
        "counting_channel_id": counting_channel.id
    }
    save_config(config)

    """
    try:
        await ctx.message.delete()
    except discord.Forbidden:
        log("💔 This bot can't delete messages")
    """

    try:
        await ctx.send(
            f"✅ Setup complete!\nLog Channel: {log_channel.mention}\nCounting Channel: {counting_channel.mention}"
        )
    except discord.Forbidden:
        log("💔 This bot couldn't send messages")


@bot.command(name="relog")
async def relog(ctx):
    if not is_admin_or_owner():
        # await ctx.send("🚫 You need admin perms to run this!")
        return

    guild_id = str(ctx.guild.id)

    if guild_id not in config:
        await ctx.send("❗ This server hasn't been set up yet! Use `!c-setup` first")
        return

    cfg = config[guild_id]

    """
    if ctx.channel.id != cfg["log_channel_id"]:
        await ctx.send("❗ Please run this command in the **log channel**!")
        return
    """

    """
    try:
        await ctx.message.delete()
    except discord.Forbidden:
        log("💔 This bot can't delete messages")
    """

    # Clear daily counts for this guild only
    keys_to_remove = [k for k in daily_counts if k.startswith(f"{guild_id}:")]
    for k in keys_to_remove:
        del daily_counts[k]

    counting_channel = await bot.fetch_channel(cfg["counting_channel_id"])
    if not counting_channel:
        await ctx.send("❓ Counting channel not found")
        return

    async for message in counting_channel.history(limit=None, oldest_first=True):
        if message.author.bot:
            continue
        if message.content.isdigit():
            msg_date = message.created_at.astimezone(tz)
            day_str = msg_date.strftime("%Y/%m/%d")
            key = f"{guild_id}:{day_str}"
            num = int(message.content)
            if key not in daily_counts or num > daily_counts[key]:
                daily_counts[key] = num

    save_data()

    # Get all unique years for this guild
    years = sorted({int(k.split(":")[1].split("/")[0]) for k in daily_counts if k.startswith(f"{guild_id}:")})

    log_channel = await bot.fetch_channel(cfg["log_channel_id"])
    if not log_channel:
        await ctx.send("❓ Log channel not found")
        return

    recent_bot_msgs = [msg async for msg in log_channel.history(limit=100) if msg.author == bot.user]

    for year in years:
        prefix = f"{guild_id}:{year}"
        year_counts = {
            k.split(":")[1]: count
            for k, count in daily_counts.items()
            if k.startswith(prefix)
        }

        log_msgs = generate_log_message(year, year_counts)

        for i, msg in enumerate(log_msgs):
            found = False
            for existing in recent_bot_msgs:
                if f"**📊 Year `{year} ({i+1})` Count Log:**" in existing.content:
                    await existing.edit(content=msg)
                    found = True
                    break
            if not found:
                await log_channel.send(msg)

    try:
        await ctx.send(
            "📤 Relog complete! Check the log channel for all updated counts"
        )
    except discord.Forbidden:
        log("💔 This bot couldn't send messages")


@bot.tree.command(
    name="helpcmd",
    description="Show a full guide about this bot"
)
async def slash_help_command(ctx):
    if not is_admin_or_owner():
        await interaction.followup.send("🚫 You need admin perms to run this!", ephemeral=True)
        return

    help_msg = """
A bot that can log progress of a counting channel in your guild!

## **-USAGE-**
Let it run and it will automatically update your logs every `5 minutes`

## **-COMMAND-**
`/helpcmd` : full guide about this bot
`/setupinfo` : view your current channel set up
`/setup <your_log_channel> <your_counting_channel>` : set each specified channel as current
`/relog` : recalculate and update all count logs in `<your_log_channel>`

## **-FORMAT-**
**📊 Year `<year>` Count Log:**
`日にち/date : 合計/sum  (5minutes change)`
`YYYY/MM/DD` : `<total_count> (<count>)`

### **-DISCLAIMER-**
This will currently detect numbers in `<your_counting_channel>` regardless of order.
We recommend using another helping bot with proper counting rules checking alongside this one.
"""
    await ctx.send(help_msg)

@bot.tree.command(
    name="setup",
    description="Set counting and log channels for this server"
)
@app_commands.describe(
    log_channel="Channel where logs are posted",
    counting_channel="Channel where users count numbers"
)
async def slash_setup(interaction: discord.Interaction, log_channel: discord.TextChannel, counting_channel: discord.TextChannel):
    if not is_admin_or_owner():
        await interaction.followup.send("🚫 You need admin perms to run this!", ephemeral=True)
        return
    
    guild_id = str(interaction.guild.id)

    if not log_channel or not counting_channel:
        guild_cfg = config.get(guild_id)

        if guild_cfg is None:
            await ctx.send("❗ This server hasn't been set up yet! Use: `!c setup #your_log_channel #your_counting_channel`")
        else:
            await ctx.send(f"📤 Log Channel: <#{guild_cfg.get('log_channel_id')}>, Counting Channel: <#{guild_cfg.get('counting_channel_id')}>")
        return

    config[guild_id] = {
        "log_channel_id": log_channel.id,
        "counting_channel_id": counting_channel.id
    }
    save_config(config)

    await interaction.response.send_message(
        f"✅ Setup complete!\nLog Channel: {log_channel.mention}\nCounting Channel: {counting_channel.mention}",
        ephemeral=True
    )

@bot.tree.command(
    name="setupinfo",
    description="Show set counting and log channels of this server"
)
async def slash_setup_info(interaction: discord.Interaction):
    if not is_admin_or_owner():
        await interaction.followup.send("🚫 You need admin perms to run this!", ephemeral=True)
        return

    guild_id = str(interaction.guild.id)
    guild_cfg = config.get(guild_id)

    if guild_cfg is None:
        await ctx.send("❗ This server hasn't been set up yet! Use: `!c setup #your_log_channel #your_counting_channel`")
    else:
        await ctx.send(f"📤 Log Channel: <#{guild_cfg.get('log_channel_id')}>, Counting Channel: <#{guild_cfg.get('counting_channel_id')}>")
    return

@bot.tree.command(
    name="relog",
    description="Recalculate and update all count logs"
)
async def slash_relog(interaction: discord.Interaction):
    if not is_admin_or_owner():
        await interaction.followup.send("🚫 You need admin perms to run this!", ephemeral=True)
        return

    await interaction.response.defer(ephemeral=True)

    guild_id = str(interaction.guild.id)

    if guild_id not in config:
        await interaction.response.send_message("❗ This server hasn't been set up yet! Use `/setup` first", ephemeral=True)
        return

    cfg = config[guild_id]

    """
    if interaction.channel.id != cfg["log_channel_id"]:
        await interaction.response.send_message("❗ Please run this command in the **log channel**!", ephemeral=True)
        return
    """

    # Clear all daily_counts only for this guild
    keys_to_remove = [k for k in daily_counts if k.startswith(f"{guild_id}:")]
    for k in keys_to_remove:
        del daily_counts[k]

    counting_channel = await bot.fetch_channel(cfg["counting_channel_id"])
    if not counting_channel:
        await interaction.followup.send("❓ Counting channel not found", ephemeral=True)
        return

    async for message in counting_channel.history(limit=None, oldest_first=True):
        if message.author.bot:
            continue
        if message.content.isdigit():
            msg_date = message.created_at.astimezone(tz)
            day_str = msg_date.strftime("%Y/%m/%d")
            key = f"{guild_id}:{day_str}"
            num = int(message.content)
            if key not in daily_counts or num > daily_counts[key]:
                daily_counts[key] = num

    save_data()

    # Get all unique years for this guild
    years = sorted({int(k.split(":")[1].split("/")[0]) for k in daily_counts if k.startswith(f"{guild_id}:")})

    log_channel = await bot.fetch_channel(cfg["log_channel_id"])
    if not log_channel:
        await interaction.followup.send("❓ Log channel not found", ephemeral=True)
        return

    recent_bot_msgs = [msg async for msg in log_channel.history(limit=100) if msg.author == bot.user]

    for year in years:
        prefix = f"{guild_id}:{year}"
        year_counts = {
            k.split(":")[1]: count
            for k, count in daily_counts.items()
            if k.startswith(prefix)
        }

        log_msgs = generate_log_message(year, year_counts)

        for i, msg in enumerate(log_msgs):
            found = False
            for existing in recent_bot_msgs:
                if f"**📊 Year `{year} ({i+1})` Count Log:**" in existing.content:
                    await existing.edit(content=msg)
                    found = True
                    break
            if not found:
                await log_channel.send(msg)

    await interaction.followup.send("📤 Relog complete! Check the log channel for all updated counts", ephemeral=True)

def generate_log_message(year, counts):
    base_header = "## **📊 Year `{}` Count Log:**\n`日にち/date : 合計/sum  (5minutes change)`\n"

    messages = []
    msg_lines = []
    data_line_count = 0
    prev_count = 0
    sorted_items = sorted(counts.items())
    part = 1

    for i, (date, count) in enumerate(sorted_items):
        parts = date.split("/")  # ['2025', '06', '16']
        month_day = "/".join(parts[1:])
        line = f"`{month_day}` : **{count}** (+{count - prev_count})"
        prev_count = count

        msg_lines.append(line)
        data_line_count += 1

        # ✨ Add separator every 5 actual data lines
        if data_line_count % 5 == 0:
            msg_lines.append(f"-# -{data_line_count}-")

        # 📦 Chunk every 50 data lines
        if data_line_count >= 50:
            header = base_header.format(f"{year} ({part})")
            messages.append(header + "\n".join(msg_lines))
            msg_lines = []
            data_line_count = 0
            part += 1

    # 📦 Add leftovers
    if msg_lines:
        header = base_header.format(f"{year} ({part})")
        messages.append(header + "\n".join(msg_lines))

    return messages


bot.run(TOKEN)
