use crate::colors;
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
    let user_to_avatar = match user {
        Some(u) => u,
        None => ctx.author().clone(),
    };

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title(&user_to_avatar.name)
            .image(user_to_avatar.face())
            .footer(footer)
            .color(colors::ROSEWATER)
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

/// Display information about a user.
#[poise::command(prefix_command, slash_command, category = "Basic")]
pub async fn userinfo(
    ctx: Context<'_>,
    #[description = "User to get info about"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = user.as_ref().unwrap_or_else(|| ctx.author());

    let mut embed = serenity::CreateEmbed::new()
        .title(format!("User Info: {}", target_user.name))
        .thumbnail(target_user.face())
        .color(colors::INFO)
        .timestamp(serenity::Timestamp::now())
        // Basic user info
        .field("Username", target_user.name.to_string(), true)
        .field("User ID", format!("`{}`", target_user.id), true)
        .field(
            "Bot Account",
            if target_user.bot { "Yes" } else { "No" },
            true,
        )
        // Account creation date
        .field(
            "Account Created",
            format!("<t:{}:F>", target_user.created_at().unix_timestamp()),
            false,
        );

    // Guild-specific information if in a guild
    if let Some(guild_id) = ctx.guild_id()
        && let Ok(member) = ctx.http().get_member(guild_id, target_user.id).await
    {
        // Join date
        if let Some(joined_at) = member.joined_at {
            embed = embed.field(
                "Joined Server",
                format!("<t:{}:F>", joined_at.unix_timestamp()),
                false,
            );
        }

        // Roles
        if !member.roles.is_empty() {
            let roles: Vec<String> = member.roles.iter().map(|r| format!("<@&{}>", r)).collect();
            let roles_str = if roles.len() > 20 {
                format!("{} and {} more", roles[..20].join(", "), roles.len() - 20)
            } else {
                roles.join(", ")
            };
            embed = embed.field(format!("Roles [{}]", member.roles.len()), roles_str, false);
        }

        // Nickname
        if let Some(nick) = &member.nick {
            embed = embed.field("Nickname", nick, true);
        }
    }

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    embed = embed.footer(footer);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Display information about the current guild/server.
#[poise::command(prefix_command, slash_command, guild_only, category = "Basic")]
pub async fn guildinfo(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.reply("This command can only be used in a server!")
                .await?;
            return Ok(());
        }
    };

    let guild = match ctx.http().get_guild(guild_id).await {
        Ok(g) => g,
        Err(_) => {
            ctx.reply("Failed to fetch guild information.").await?;
            return Ok(());
        }
    };

    let mut embed = serenity::CreateEmbed::new()
        .title(format!("Server Info: {}", guild.name))
        .color(colors::SUCCESS)
        .timestamp(serenity::Timestamp::now());

    // Server icon
    if let Some(icon_url) = guild.icon_url() {
        embed = embed.thumbnail(icon_url);
    }

    // Basic info
    embed = embed
        .field("Server ID", format!("`{}`", guild.id), true)
        .field("Owner", format!("<@{}>", guild.owner_id), true)
        // Creation date
        .field(
            "Created",
            format!("<t:{}:F>", guild.id.created_at().unix_timestamp()),
            false,
        );

    // Channel counts
    let channels = ctx.http().get_channels(guild_id).await.unwrap_or_default();
    let text_channels = channels
        .iter()
        .filter(|c| matches!(c.kind, serenity::ChannelType::Text))
        .count();
    let voice_channels = channels
        .iter()
        .filter(|c| matches!(c.kind, serenity::ChannelType::Voice))
        .count();

    embed = embed.field("Text Channels", format!("{}", text_channels), true);
    embed = embed.field("Voice Channels", format!("{}", voice_channels), true);

    // Boost info
    let boost_level = match guild.premium_tier {
        serenity::model::guild::PremiumTier::Tier0 => 0,
        serenity::model::guild::PremiumTier::Tier1 => 1,
        serenity::model::guild::PremiumTier::Tier2 => 2,
        serenity::model::guild::PremiumTier::Tier3 => 3,
        _ => 0,
    };
    embed = embed.field("Boost Level", format!("Level {}", boost_level), true);
    embed = embed.field(
        "Boosts",
        format!("{}", guild.premium_subscription_count.unwrap_or(0)),
        true,
    );

    // Verification level
    embed = embed.field(
        "Verification Level",
        format!("{:?}", guild.verification_level),
        true,
    );

    // Description
    if let Some(description) = &guild.description
        && !description.is_empty()
    {
        embed = embed.field("Description", description, false);
    }

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    embed = embed.footer(footer);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
