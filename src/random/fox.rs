use crate::env::FOOTER_URL;
use crate::{Context, Error, HTTP_CLIENT, colors};

use poise::serenity_prelude as serenity;

#[poise::command(prefix_command, slash_command, category = "Random")]
pub async fn fox(ctx: Context<'_>) -> Result<(), Error> {
    let resp = HTTP_CLIENT
        .get()
        .unwrap()
        .get("https://randomfox.ca/floof")
        .send()
        .await?;

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let json: serde_json::Value = resp.json().await?; // Parse JSON response
    let image_url = json["image"].as_str().unwrap_or(""); // Extract the image field
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title("what does the fox say?")
            .description("xdd")
            .image(image_url) // Use the correct image URL
            .footer(footer)
            .color(colors::PEACH)
            // Add a timestamp for the current time
            // This also accepts a rfc3339 Timestamp
            .timestamp(serenity::model::Timestamp::now());

        poise::CreateReply::default().embed(embed)
    };

    ctx.send(reply).await?;

    Ok(())
}
