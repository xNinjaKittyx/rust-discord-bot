use crate::env::FOOTER_URL;
use crate::{Context, Error, HTTP_CLIENT};

use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use songbird::input::YoutubeDl;
use strsim::jaro_winkler;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Mirror {
    pub mirror_u_r_l: String,
    pub priority: u16,
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Theme {
    pub theme_type: String,
    pub theme_name: String,
    pub mirror: Mirror,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Roulette {
    pub name: String,
    pub themes: Vec<Theme>,
}

#[poise::command(prefix_command, slash_command, category = "Anime")]
pub async fn guess(ctx: Context<'_>) -> Result<(), Error> {
    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));

    let guild_id = ctx.guild_id().unwrap();
    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let http_client = HTTP_CLIENT.get().unwrap();
        let resp = http_client
            .get("https://themes.moe/api/roulette")
            .send()
            .await?;
        let results: Result<Roulette, serde_json::Error> =
            serde_json::from_str(resp.text().await.unwrap().as_str());
        let result = results.unwrap();
        let src = YoutubeDl::new(
            http_client.clone(),
            result.themes[0].mirror.mirror_u_r_l.clone(),
        );

        let answer = format!(
            "{} - {} - {}",
            result.name, result.themes[0].theme_name, result.themes[0].theme_type
        );
        log::info!("{}", answer);

        let track = handler.play(src.clone().into());
        let reply = {
            let embed = serenity::CreateEmbed::new()
                .description("Guess the OP! Easy Mode - Playing through the whole song. You have 2 minutes to guess!")
                .footer(footer.clone())
                .timestamp(serenity::model::Timestamp::now());
            poise::CreateReply::default().embed(embed)
        };
        ctx.send(reply).await?;

        while let Some(msg) = serenity::MessageCollector::new(ctx)
            .channel_id(ctx.channel_id())
            .timeout(std::time::Duration::from_secs(120))
            .await
        {
            let winkler_score = jaro_winkler(&msg.content, result.name.as_str()) as f64;
            log::info!("Winkler Score for {} is {}", msg.content, winkler_score);
            if msg.content == result.name || winkler_score > 0.85 {
                msg.reply(
                    ctx,
                    format!("You guessed it correctly! The answer was {}", answer),
                )
                .await?;
                break;
            }
        }
        let _ = track.stop();
    } else {
        let reply = {
            let embed = serenity::CreateEmbed::new()
                .description("I'm not in a channel.")
                .footer(footer)
                .timestamp(serenity::model::Timestamp::now());
            poise::CreateReply::default().embed(embed)
        };
        ctx.send(reply).await?;
    }
    Ok(())
}
