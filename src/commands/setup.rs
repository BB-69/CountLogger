use serenity::prelude::*;
use serenity::model::application::*;
use serenity::model::prelude::*;
use serenity::builder::*;

use crate::data::{BotData, load_guild_data, save_guild_data};
use crate::utils::check_admin;

pub fn register() -> CreateCommand {
    CreateCommand::new("setup")
        .description("Setup bundled commands")
}

pub async fn execute(ctx: Context, command: CommandInteraction, bot_data: &BotData) {
    if !check_admin(&ctx, &command).await { return; }

    if let Some(guild_id) = command.guild_id {
        let guild_id_u64 = guild_id.get();
        let mut guild_data = load_guild_data(guild_id_u64);

        let page = 1;
        let total = 3;

        command.create_response(&ctx.http, |r| {
            r.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|msg| {
                    msg.ephemeral(true)
                        .add_embed(build_page_embed(page, total))
                        .components(|c| {
                            for row in build_page_components(page, total) {
                                c.add_action_row(row);
                            }
                            c
                        })
                })
        }).await.unwrap();

        {
            let mut guilds = bot_data.guilds.lock().await;
            guilds.insert(guild_id_u64, guild_data.clone());
        }
        save_guild_data(guild_id_u64, &guild_data);
    }
}

pub async fn handle_component(ctx: &Context, component: &MessageComponentInteraction) {
    let parts: Vec<&str> = component.data.custom_id.split(':').collect();
    if parts.len() != 3 || parts[0] != "setup" || parts[1] != "page" {
        return;
    }

    let mut page: usize = parts[2].parse().unwrap_or(1);
    let total = 3;

    match component.data.custom_id.as_str() {
        "setup:page:back" => {
            if page > 1 { page -= 1; }
        }
        "setup:page:next" => {
            if page < total { page += 1; }
        }
        "setup:page:confirm" => {
            page = total;
        }
        _ => {}
    }

    component.create_interaction_response(&ctx.http, |r| {
        r.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|msg| {
                msg.set_embeds(vec![build_page_embed(page, total)])
                    .components(|c| {
                        for row in build_page_components(page, total) {
                            c.add_action_row(row);
                        }
                        c
                    })
            })
    }).await.unwrap();
}

fn build_page_embed(page: usize, total: usize) -> CreateEmbed {
    let mut embed = CreateEmbed::default();

    match page {
        1 => {
            embed.title("Setup - Page 1")
                .description("Pick a UTC (dropdown coming soon 💙)")
                .footer(|f| f.text(format!("Page {}/{}", page, total)));
        }
        2 => {
            embed.title("Setup - Page 2")
                .description("Counting Channel ID + Log Channel ID inputs")
                .footer(|f| f.text(format!("Page {}/{}", page, total)));
        }
        3 => {
            embed.title("Setup - Page 3")
                .description("✅ Setup Done! Click Confirm to save.")
                .footer(|f| f.text(format!("Page {}/{}", page, total)));
        }
        _ => {}
    }

    embed
}

fn build_page_components(page: usize, total: usize) -> Vec<CreateActionRow> {
    let mut rows = Vec::new();
    let mut row = CreateActionRow::default();

    // Back button
    row.add_button(CreateButton::new(format!("setup:page:{}", "back"))
        .label("⬅️ Back")
        .style(ButtonStyle::Secondary)
        .disabled(page == 1));

    // Next / Confirm button
    if page < total {
        row.add_button(CreateButton::new(format!("setup:page:{}", "next"))
            .label("➡️ Next")
            .style(ButtonStyle::Primary));
    } else {
        row.add_button(CreateButton::new(format!("setup:page:{}", "confirm"))
            .label("✅ Confirm")
            .style(ButtonStyle::Success));
    }

    rows.push(row);
    rows
}

