use crate::env::FOOTER_URL;
use crate::{Context, Error, HTTP_CLIENT};

use poise::serenity_prelude as serenity;

#[poise::command(prefix_command, slash_command)]
pub async fn shibe(ctx: Context<'_>) -> Result<(), Error> {
    let resp = HTTP_CLIENT
        .get()
        .unwrap()
        .get("https://shibe.online/api/shibes")
        .send()
        .await?;

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let image = resp.text().await?;
    let links: Vec<String> = serde_json::from_str(image.as_str())?;
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title("wow")
            .description("## such doge\n       # much impress\n   ### very fluff\n    so cool")
            .image(&links[0])
            .footer(footer)
            // Add a timestamp for the current time
            // This also accepts a rfc3339 Timestamp
            .timestamp(serenity::model::Timestamp::now());

        poise::CreateReply::default().embed(embed)
    };

    ctx.send(reply).await?;

    Ok(())
}
