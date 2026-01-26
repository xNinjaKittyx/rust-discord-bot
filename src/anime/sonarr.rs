use crate::anime::sonarr_serde;
use crate::env::{FOOTER_URL, SONARR_API_KEY, SONARR_URL};
use crate::{Context, Error, HTTP_CLIENT, colors};

use chrono_tz::America::Los_Angeles;

use poise::serenity_prelude as serenity;
use reqwest;

pub async fn paginate_embed<U, E>(
    ctx: poise::Context<'_, U, E>,
    pages: &[serenity::CreateEmbed],
) -> Result<(), serenity::Error> {
    if pages.len() == 1 {
        // Then we don't really need paginate, and we should just return.
        let reply = { poise::CreateReply::default().embed(pages[0].clone()) };

        ctx.send(reply).await?;
        return Ok(());
    }

    // Define some unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);
    // Send the embed with the first page as content
    let reply = {
        let components = serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(&prev_button_id).emoji('◀'),
            serenity::CreateButton::new(&next_button_id).emoji('▶'),
        ]);

        poise::CreateReply::default()
            .embed(pages[0].clone())
            .components(vec![components])
    };

    ctx.send(reply).await?;

    // Loop through incoming interactions with the navigation buttons
    let mut current_page = 0;
    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        // We defined our button IDs to start with `ctx_id`. If they don't, some other command's
        // button was pressed
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        // Timeout when no navigation button has been pressed for 24 hours
        .timeout(std::time::Duration::from_secs(3600 * 24))
        .await
    {
        // Depending on which button was pressed, go to next or previous page
        if press.data.custom_id == next_button_id {
            current_page += 1;
            if current_page >= pages.len() {
                current_page = 0;
            }
        } else if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(pages.len() - 1);
        } else {
            // This is an unrelated button interaction
            continue;
        }

        // Update the message with the new page contents
        press
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(pages[current_page].clone()),
                ),
            )
            .await?;
    }

    Ok(())
}

pub async fn paginate<U, E>(
    ctx: poise::Context<'_, U, E>,
    pages: &[&str],
    title: &str,
) -> Result<(), serenity::Error> {
    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let embed = serenity::CreateEmbed::default()
        .title(title)
        .footer(footer)
        .timestamp(serenity::model::Timestamp::now());

    if pages.len() == 1 {
        // Then we don't really need paginate, and we should just return.
        let reply = { poise::CreateReply::default().embed(embed.clone().description(pages[0])) };

        ctx.send(reply).await?;
        return Ok(());
    }

    // Define some unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);
    // Send the embed with the first page as content
    let reply = {
        let components = serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(&prev_button_id).emoji('◀'),
            serenity::CreateButton::new(&next_button_id).emoji('▶'),
        ]);

        poise::CreateReply::default()
            .embed(embed.clone().description(pages[0]))
            .components(vec![components])
    };

    ctx.send(reply).await?;

    // Loop through incoming interactions with the navigation buttons
    let mut current_page = 0;
    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        // We defined our button IDs to start with `ctx_id`. If they don't, some other command's
        // button was pressed
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        // Timeout when no navigation button has been pressed for 24 hours
        .timeout(std::time::Duration::from_secs(3600 * 24))
        .await
    {
        // Depending on which button was pressed, go to next or previous page
        if press.data.custom_id == next_button_id {
            current_page += 1;
            if current_page >= pages.len() {
                current_page = 0;
            }
        } else if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(pages.len() - 1);
        } else {
            // This is an unrelated button interaction
            continue;
        }

        // Update the message with the new page contents
        press
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(embed.clone().description(pages[current_page])),
                ),
            )
            .await?;
    }

    Ok(())
}

async fn execute_sonarr_request(endpoint: &str) -> Result<String, reqwest::Error> {
    let resp = HTTP_CLIENT
        .get()
        .unwrap()
        .get(format!("{}/api/v3/{}", &*SONARR_URL, endpoint))
        .header("X-Api-Key", &*SONARR_API_KEY)
        .send()
        .await?;

    let json_string = resp.text().await?;
    log::info!("Sonarr returned {}", json_string);
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

async fn parse_history(daily: bool) -> Result<Vec<sonarr_serde::History>, serde_json::Error> {
    let today = chrono::Utc::now();
    let yesterday = if daily {
        today - chrono::Duration::days(1)
    } else {
        today - chrono::Duration::days(7)
    };
    let request_uri = format!(
        "history/since?date={}T00:00:00Z&eventType=grabbed",
        yesterday.format("%Y-%m-%d")
    );
    let json_string = execute_sonarr_request(&request_uri).await.unwrap();
    // Deserialize the JSON string into a Value
    let results: Result<Vec<sonarr_serde::History>, serde_json::Error> =
        serde_json::from_str(json_string.as_str());
    let shows = results.unwrap();
    Ok(shows)
}

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("follow", "schedule", "recent", "showall"),
    subcommand_required,
    category = "Anime"
)]
pub async fn sonarr(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Anime")]
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

    let pages = vec![description.as_str()];

    paginate(ctx, &pages, "List of Downloads Today (PST)").await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Anime")]
pub async fn schedule(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Anime")]
pub async fn showall(ctx: Context<'_>) -> Result<(), Error> {
    let shows = parse_shows().await?;
    let mut embeds: Vec<serenity::CreateEmbed> = Vec::new();
    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));

    for show in shows {
        let embed = serenity::CreateEmbed::new()
            .title(&show.title)
            .description(&show.overview)
            .fields(vec![
                (
                    "Episodes",
                    format!(
                        "{}/{} - {:.2}%\n",
                        show.statistics.episode_file_count,
                        show.statistics.episode_count,
                        show.statistics.percent_of_episodes
                    ),
                    true,
                ),
                ("Status", show.status.to_string(), true),
                ("Episode Length", show.runtime.to_string(), false),
                ("Year", show.year.to_string(), false),
                (
                    "Release Groups",
                    show.statistics.release_groups.join(", ").to_string(),
                    false,
                ),
            ])
            .thumbnail(&show.images[1].remote_url)
            .image(&show.images[0].remote_url)
            .color(colors::SKY)
            .footer(footer.clone())
            // Add a timestamp for the current time
            // This also accepts a rfc3339 Timestamp
            .timestamp(serenity::model::Timestamp::now());
        embeds.push(embed.clone())
    }

    paginate_embed(ctx, &embeds).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Anime")]
pub async fn recent(ctx: Context<'_>, weekly: bool) -> Result<(), Error> {
    let history: Vec<sonarr_serde::History> = parse_history(!weekly).await?;

    let mut description = String::new();
    let mut pages: Vec<String> = Vec::new();

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
        if description.len() > 3000 {
            pages.push(description.clone());
            description.clear();
        }
    }

    if !description.is_empty() {
        pages.push(description);
    }
    let new_pages: Vec<&str> = pages.iter().map(String::as_str).collect();
    paginate(ctx, &new_pages, "List of Downloads Today (PST)").await?;
    Ok(())
}
