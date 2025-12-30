use crate::env::FOOTER_URL;
use crate::{Context, Error, HTTP_CLIENT};

use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct KanjiResult {
    grade: u16,
    heisig_en: String,
    jlpt: Option<u16>,
    kanji: String,
    kun_readings: Vec<String>,
    meanings: Vec<String>,
    name_readings: Vec<String>,
    notes: Vec<String>,
    on_readings: Vec<String>,
    stroke_count: u16,
}

#[poise::command(prefix_command, slash_command, category = "Utility")]
pub async fn kanji(
    ctx: Context<'_>,
    #[description = "Japanese kanji character to lookup"] kanji: String,
) -> Result<(), Error> {
    let url = format!("https://kanjiapi.dev/v1/kanji/{}", kanji);
    log::info!("Sending Request to {}", url);
    let resp = HTTP_CLIENT.get().unwrap().get(url).send().await?;

    let text = resp.text().await?;
    log::info!("Kanji returned {}", text);

    let result: KanjiResult = serde_json::from_str(text.as_str())?;

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title(&result.kanji)
            .description(&result.meanings.join(", "))
            .fields(vec![
                ("School Grade", result.grade.to_string(), true),
                ("JLPT", result.jlpt.unwrap_or(0).to_string(), true),
                ("訓読み", result.kun_readings.join(", "), false),
                ("音読み", result.on_readings.join(", "), false),
                ("Name Readings", result.name_readings.join(", "), false),
                (
                    "Jisho",
                    format!("[Link](https://jisho.org/search/{}%23kanji)", result.kanji),
                    false,
                ),
            ])
            .footer(footer)
            // Add a timestamp for the current time
            // This also accepts a rfc3339 Timestamp
            .timestamp(serenity::model::Timestamp::now());

        poise::CreateReply::default().embed(embed)
    };

    ctx.send(reply).await?;
    Ok(())
}
