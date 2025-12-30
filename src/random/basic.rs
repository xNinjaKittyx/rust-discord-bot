use crate::env::FOOTER_URL;
use crate::{Context, Error};

use poise::serenity_prelude as serenity;


/// Ping the bot to see if it's alive.
#[poise::command(prefix_command, slash_command, category = "Basic")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("Pong!").await?;
    Ok(())
}


/// Bing Bong
#[poise::command(prefix_command, slash_command, category = "Basic")]
pub async fn bing(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("Bong!").await?;
    Ok(())
}

/// Get the avatar of a user or the command invoker.
#[poise::command(prefix_command, slash_command, category = "Basic")]
pub async fn avatar(
    ctx: Context<'_>,
    #[description = "Name of User"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user_to_avatar;
    match user {
        Some(u) => user_to_avatar = u,
        None => user_to_avatar = ctx.author().clone(),
    }

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title(&user_to_avatar.name)
            .image(user_to_avatar.face())
            .footer(footer)
            // Add a timestamp for the current time
            // This also accepts a rfc3339 Timestamp
            .timestamp(serenity::model::Timestamp::now());

        poise::CreateReply::default().embed(embed)
    };
    ctx.send(reply).await?;

    Ok(())
}

/// Display help information about commands.
#[poise::command(slash_command, prefix_command, category = "Basic")]
pub async fn help(ctx: Context<'_>, command: Option<String>) -> Result<(), Error> {
    let configuration = poise::builtins::HelpConfiguration {
        // [configure aspects about the help message here]
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), configuration).await?;
    Ok(())
}

/// Display the bot's uptime.
#[poise::command(prefix_command, slash_command, category = "Basic")]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let start_time = crate::START_TIME.get();

    if let Some(start) = start_time {
        let elapsed = start.elapsed();
        let seconds = elapsed.as_secs();

        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        let uptime_str = if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, secs)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        };

        ctx.reply(format!("🕐 Bot uptime: {}", uptime_str)).await?;
    } else {
        ctx.reply("Uptime information not available.").await?;
    }

    Ok(())
}
