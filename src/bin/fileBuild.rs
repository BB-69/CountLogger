use std::fs::{self, OpenOptions};
use std::path::Path;
use std::io::{self, Write};

fn main() {
    println!("Creating 'data.json' ..."); {
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

    println!("Creating '.env' ..."); {
        let path = ".env";

        let mut answer = String::new();
        answer = loop {
            let ans = if answer.is_empty() {
                get_input(&format!("'{}' already exists! Edit file anyway? (Y/N): ", path))
            } else {
                get_input("Edit file anyway? (Y/N): ")
            }.to_uppercase();

            if ans == "Y" || ans == "N" {
                break ans;
            } else {
                println!("Invalid answer");
            }
        };

        if answer == "Y" {
            let discord_token = get_input("Enter your DISCORD_TOKEN: ");
            let test_guild_id = get_input("Enter your TEST_GUILD_ID: ");

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(".env")
                .unwrap();

            writeln!(file, "DISCORD_TOKEN={}", discord_token).unwrap();
            writeln!(file, "TEST_GUILD_ID={}", test_guild_id).unwrap();

            println!("'{}' created!", path);
        }
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
