import os
import json
import subprocess

print("==== Welcome to CountLogger Setup Wizard~! ====\n")

# 1. Ask for Token + Guild ID
if not os.path.exists(".env"):
    print("🔐 Let's get your bot connected~")
    token = input("📝 Enter your Discord Bot Token: ").strip()
    guild_id = input("📝 Enter your Guild (Server) ID: ").strip()

    with open(".env", "w") as f:
        f.write(f"DISCORD_TOKEN={token}\n")
        f.write(f"TEST_GUILD_ID={guild_id}\n")
    print("✅ .env file created!")

else:
    print("📁 .env already exists. Skipping creation...")

# 2. Create data.json if not exists
if not os.path.exists("data.json"):
    print("📊 Creating empty data.json...")
    with open("data.json", "w") as f:
        json.dump({}, f, indent=4)
    print("✅ data.json ready to go!")
else:
    print("📁 data.json already exists. Skipping creation...")

# 3. Create config.json if not exists
if not os.path.exists("config.json"):
    print("⚙️ Creating empty config.json...")
    with open("config.json", "w") as f:
        json.dump({}, f, indent=4)
    print("✅ config.json created!")
else:
    print("📁 config.json already exists. Skipping creation...")

# 4. Offer to install dependencies
if os.path.exists("requirements.txt"):
    user_input = input("📦 Want to install requirements now? (y/n): ").lower()
    if user_input == "y":
        subprocess.call(["pip", "install", "-r", "requirements.txt"])
    else:
        print("⏩ Skipped requirements install.")

print("\n🌟 All done! You're all set to run the bot 💙")
