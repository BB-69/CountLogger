use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

fn main() {
    println!("Creating 'data.json' ...");
    {
        let path = "src/data/data.json";
        let path_obj = Path::new(path);

        if path_obj.exists() {
            println!("'{}' already exists!", path);
        } else {
            if let Some(parent) = path_obj.parent() {
                fs::create_dir_all(parent).expect("Failed to create parent directories");
            }

            fs::write(path, "{}").expect("Failed to create file");
            println!("'{}' created!", path);
        }
    }

    println!("Creating '.env' ...");
    {
        let path = ".env";

        let discord_token = get_input("Enter your DISCORD_TOKEN: ");
        let test_guild_id = get_input("Enter your TEST_GUILD_ID: ");
        let bot_owner_id = get_input("Enter your BOT_OWNER_ID: ");

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(".env")
            .unwrap();

        writeln!(file, "DISCORD_TOKEN={}", discord_token).unwrap();
        writeln!(file, "TEST_GUILD_ID={}", test_guild_id).unwrap();
        writeln!(file, "BOT_OWNER_ID={}", bot_owner_id).unwrap();

        println!("'{}' created!", path);
    }

    println!("✅ Finished building files!");
}

fn get_input(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
