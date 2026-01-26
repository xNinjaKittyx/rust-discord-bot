use crate::{Context, Error, colors};
use poise::serenity_prelude as serenity;
use std::sync::RwLock;

// Store the current activity to persist across status changes
static CURRENT_ACTIVITY: RwLock<Option<serenity::ActivityData>> = RwLock::new(None);

/// Set the bot's status
#[poise::command(prefix_command, slash_command, owners_only, category = "Utility")]
pub async fn setstatus(
    ctx: Context<'_>,
    #[description = "Status to set (online/idle/dnd/invisible)"] status: String,
) -> Result<(), Error> {
    let status_lower = status.to_lowercase();
    let new_status = match status_lower.as_str() {
        "online" => serenity::OnlineStatus::Online,
        "idle" => serenity::OnlineStatus::Idle,
        "dnd" => serenity::OnlineStatus::DoNotDisturb,
        "invisible" => serenity::OnlineStatus::Invisible,
        _ => {
            ctx.send(
                poise::CreateReply::default()
                    .embed(
                        serenity::CreateEmbed::new()
                            .title("Invalid Status")
                            .description("Valid statuses: `online`, `idle`, `dnd`, `invisible`")
                            .color(colors::ERROR)
                            .timestamp(serenity::model::Timestamp::now()),
                    )
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // Preserve the current activity when changing status
    let current_activity = CURRENT_ACTIVITY.read().unwrap().clone();
    ctx.serenity_context()
        .set_presence(current_activity, new_status);

    ctx.send(
        poise::CreateReply::default()
            .embed(
                serenity::CreateEmbed::new()
                    .title("Status Updated")
                    .description(format!("Bot status set to: **{}**", status_lower))
                    .color(colors::SUCCESS)
                    .timestamp(serenity::model::Timestamp::now()),
            )
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// Set the bot's avatar
#[poise::command(prefix_command, slash_command, owners_only, category = "Utility")]
pub async fn setavatar(
    ctx: Context<'_>,
    #[description = "Link to the new avatar image"] link: String,
) -> Result<(), Error> {
    // Download the image
    let client = reqwest::Client::new();
    let response = client.get(&link).send().await?;

    if !response.status().is_success() {
        ctx.send(
            poise::CreateReply::default()
                .embed(
                    serenity::CreateEmbed::new()
                        .title("Error")
                        .description("Failed to download image from the provided link")
                        .color(colors::ERROR)
                        .timestamp(serenity::model::Timestamp::now()),
                )
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let bytes = response.bytes().await?;

    // Set the avatar
    let attachment = serenity::CreateAttachment::bytes(bytes.to_vec(), "avatar.png");
    let mut current_user = ctx.serenity_context().http.get_current_user().await?;
    current_user
        .edit(
            ctx.serenity_context(),
            serenity::EditProfile::new().avatar(&attachment),
        )
        .await?;

    ctx.send(
        poise::CreateReply::default()
            .embed(
                serenity::CreateEmbed::new()
                    .title("Avatar Updated")
                    .description("Bot avatar has been successfully updated")
                    .thumbnail(link)
                    .color(colors::SUCCESS)
                    .timestamp(serenity::model::Timestamp::now()),
            )
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// Set the bot's banner
#[poise::command(prefix_command, slash_command, owners_only, category = "Utility")]
pub async fn setbanner(
    ctx: Context<'_>,
    #[description = "Link to the new banner image"] link: String,
) -> Result<(), Error> {
    // Download the image
    let client = reqwest::Client::new();
    let response = client.get(&link).send().await?;

    if !response.status().is_success() {
        ctx.send(
            poise::CreateReply::default()
                .embed(
                    serenity::CreateEmbed::new()
                        .title("Error")
                        .description("Failed to download image from the provided link")
                        .color(colors::ERROR)
                        .timestamp(serenity::model::Timestamp::now()),
                )
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let bytes = response.bytes().await?;

    // Set the banner
    let attachment = serenity::CreateAttachment::bytes(bytes.to_vec(), "banner.png");
    let mut current_user = ctx.serenity_context().http.get_current_user().await?;
    current_user
        .edit(
            ctx.serenity_context(),
            serenity::EditProfile::new().banner(&attachment),
        )
        .await?;

    ctx.send(
        poise::CreateReply::default()
            .embed(
                serenity::CreateEmbed::new()
                    .title("Banner Updated")
                    .description("Bot banner has been successfully updated")
                    .image(link)
                    .color(colors::SUCCESS)
                    .timestamp(serenity::model::Timestamp::now()),
            )
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// Set the bot's activity
#[poise::command(prefix_command, slash_command, owners_only, category = "Utility")]
pub async fn setactivity(
    ctx: Context<'_>,
    #[description = "Activity text (empty to reset, prefix with Playing/Listening/Watching/Competing/Streaming)"]
    #[rest]
    text: Option<String>,
) -> Result<(), Error> {
    match text.as_ref() {
        None => {
            // Reset presence and clear stored activity
            *CURRENT_ACTIVITY.write().unwrap() = None;
            ctx.serenity_context()
                .set_presence(None, serenity::OnlineStatus::Online);

            ctx.send(
                poise::CreateReply::default()
                    .embed(
                        serenity::CreateEmbed::new()
                            .title("Activity Reset")
                            .description("Bot activity has been cleared")
                            .color(colors::SUCCESS)
                            .timestamp(serenity::model::Timestamp::now()),
                    )
                    .ephemeral(true),
            )
            .await?;
        }
        Some(t) if t.trim().is_empty() => {
            // Reset presence and clear stored activity
            *CURRENT_ACTIVITY.write().unwrap() = None;
            ctx.serenity_context()
                .set_presence(None, serenity::OnlineStatus::Online);

            ctx.send(
                poise::CreateReply::default()
                    .embed(
                        serenity::CreateEmbed::new()
                            .title("Activity Reset")
                            .description("Bot activity has been cleared")
                            .color(colors::SUCCESS)
                            .timestamp(serenity::model::Timestamp::now()),
                    )
                    .ephemeral(true),
            )
            .await?;
        }
        Some(text) => {
            let parts: Vec<&str> = text.splitn(2, ' ').collect();
            let first_word = parts[0];

            let activity = match first_word {
                "Playing" => {
                    let name = parts.get(1).unwrap_or(&"").to_string();
                    serenity::ActivityData::playing(name)
                }
                "Listening" => {
                    let name = parts.get(1).unwrap_or(&"").to_string();
                    serenity::ActivityData::listening(name)
                }
                "Watching" => {
                    let name = parts.get(1).unwrap_or(&"").to_string();
                    serenity::ActivityData::watching(name)
                }
                "Competing" => {
                    let name = parts.get(1).unwrap_or(&"").to_string();
                    serenity::ActivityData::competing(name)
                }
                "Streaming" => {
                    // For streaming, the last value should be the link
                    if parts.len() < 2 {
                        ctx.send(
                            poise::CreateReply::default()
                                .embed(
                                    serenity::CreateEmbed::new()
                                        .title("Error")
                                        .description(
                                            "Streaming requires format: `Streaming <name> <url>`",
                                        )
                                        .color(colors::ERROR)
                                        .timestamp(serenity::model::Timestamp::now()),
                                )
                                .ephemeral(true),
                        )
                        .await?;
                        return Ok(());
                    }

                    let rest = parts[1];
                    let rest_parts: Vec<&str> = rest.rsplitn(2, ' ').collect();

                    if rest_parts.len() < 2 {
                        ctx.send(
                            poise::CreateReply::default()
                                .embed(
                                    serenity::CreateEmbed::new()
                                        .title("Error")
                                        .description(
                                            "Streaming requires format: `Streaming <name> <url>`",
                                        )
                                        .color(colors::ERROR)
                                        .timestamp(serenity::model::Timestamp::now()),
                                )
                                .ephemeral(true),
                        )
                        .await?;
                        return Ok(());
                    }

                    let url = rest_parts[0];
                    let name = rest_parts[1];

                    serenity::ActivityData::streaming(name, url)?
                }
                _ => {
                    // Use custom activity
                    serenity::ActivityData::custom(text)
                }
            };

            // Store and set the activity
            *CURRENT_ACTIVITY.write().unwrap() = Some(activity.clone());
            ctx.serenity_context()
                .set_presence(Some(activity.clone()), serenity::OnlineStatus::Online);

            let activity_desc = match activity.kind {
                serenity::ActivityType::Playing => format!("Playing **{}**", activity.name),
                serenity::ActivityType::Listening => format!("Listening to **{}**", activity.name),
                serenity::ActivityType::Watching => format!("Watching **{}**", activity.name),
                serenity::ActivityType::Competing => format!("Competing in **{}**", activity.name),
                serenity::ActivityType::Streaming => {
                    format!("Streaming **{}**", activity.name)
                }
                serenity::ActivityType::Custom => {
                    format!("**{}**", activity.state.unwrap_or_default())
                }
                _ => activity.name,
            };

            ctx.send(
                poise::CreateReply::default()
                    .embed(
                        serenity::CreateEmbed::new()
                            .title("Activity Updated")
                            .description(format!("Bot activity set to: {}", activity_desc))
                            .color(colors::SUCCESS)
                            .timestamp(serenity::model::Timestamp::now()),
                    )
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}
