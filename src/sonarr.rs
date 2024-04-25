use crate::sonarr_serde;
use crate::{Context, Error, FOOTER_URL, HTTP_CLIENT};

use std::sync::LazyLock;

use chrono_tz::America::Los_Angeles;

use poise::serenity_prelude as serenity;
use reqwest;

static SONARR_URL: LazyLock<String> = LazyLock::new(|| std::env::var("SONARR_URL").unwrap());

static SONARR_API_KEY: LazyLock<String> =
    LazyLock::new(|| std::env::var("SONARR_API_KEY").unwrap());

async fn execute_sonarr_request(endpoint: &str) -> Result<String, reqwest::Error> {
    let resp = HTTP_CLIENT
        .get_or_init(|| reqwest::Client::new())
        .get(format!("{}/api/v3/{}", &*SONARR_URL, endpoint))
        .header("X-Api-Key", &*SONARR_API_KEY)
        .send()
        .await?;

    let json_string = resp.text().await?;
    Ok(json_string)
}

async fn parse_shows() -> Result<Vec<sonarr_serde::Show>, serde_json::Error> {
    let json_string = execute_sonarr_request("series").await.unwrap();

    // Deserialize the JSON string into a Value
    let results: Result<Vec<sonarr_serde::Show>, serde_json::Error> =
        serde_json::from_str(json_string.as_str());
    let shows = results.unwrap();
    Ok(shows)
}

async fn parse_history() -> Result<Vec<sonarr_serde::History>, serde_json::Error> {
    let json_string =
        execute_sonarr_request("history/since?date=2024-04-19T00%3A00%3A00Z&eventType=grabbed")
            .await
            .unwrap();
    // Deserialize the JSON string into a Value
    let results: Result<Vec<sonarr_serde::History>, serde_json::Error> =
        serde_json::from_str(json_string.as_str());
    let shows = results.unwrap();
    Ok(shows)
}

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("follow", "schedule", "recent")
)]
pub async fn sonarr(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn follow(ctx: Context<'_>) -> Result<(), Error> {
    let shows = parse_shows().await?;

    let mut description = String::new();
    description.push_str("```\n");
    for show in shows.iter() {
        if show.status == "continuing" {
            description.push_str(
                format!(
                    "{} - {}/{} - {:.2}%\n",
                    show.title,
                    show.statistics.episode_file_count,
                    show.statistics.episode_count,
                    show.statistics.percent_of_episodes
                )
                .as_str(),
            );
        }
    }
    description.push_str("```\n");

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title("List of Automated Downloads")
            .description(description)
            // .image("attachment://ferris_eyes.png")
            // .fields(fields_vector)
            .footer(footer)
            // Add a timestamp for the current time
            // This also accepts a rfc3339 Timestamp
            .timestamp(serenity::model::Timestamp::now());

        poise::CreateReply::default().embed(embed)
    };

    ctx.send(reply).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn schedule(ctx: Context<'_>) -> Result<(), Error> {
    let history: Vec<sonarr_serde::History> = parse_history().await?;

    let mut description = String::new();

    for his in history.iter() {
        description.push_str(
            format!(
                "**{}**",
                his.date
                    .with_timezone(&Los_Angeles)
                    .format("%Y/%m/%d %H:%M")
            )
            .as_str(),
        );
        description.push_str("\n```");
        description.push_str(his.source_title.as_str());
        description.push_str("```\n");
    }

    let footer = serenity::CreateEmbedFooter::new("Powered by urfmode.moe");
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title("List of Downloads Today")
            .description(description)
            // .image("attachment://ferris_eyes.png")
            // .fields(fields_vector)
            .footer(footer)
            // Add a timestamp for the current time
            // This also accepts a rfc3339 Timestamp
            .timestamp(serenity::model::Timestamp::now());

        poise::CreateReply::default().embed(embed)
    };

    ctx.send(reply).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn recent(ctx: Context<'_>) -> Result<(), Error> {
    let history: Vec<sonarr_serde::History> = parse_history().await?;

    let mut description = String::new();

    for his in history.iter() {
        description.push_str(
            format!(
                "**{}**",
                his.date
                    .with_timezone(&Los_Angeles)
                    .format("%Y/%m/%d %H:%M")
            )
            .as_str(),
        );
        description.push_str("\n```");
        description.push_str(his.source_title.as_str());
        description.push_str("```\n");
    }

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title("List of Downloads Today")
            .description(description)
            // .image("attachment://ferris_eyes.png")
            // .fields(fields_vector)
            .footer(footer)
            // Add a timestamp for the current time
            // This also accepts a rfc3339 Timestamp
            .timestamp(serenity::model::Timestamp::now());

        poise::CreateReply::default().embed(embed)
    };

    ctx.send(reply).await?;
    Ok(())
}
