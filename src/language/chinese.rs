use crate::env::FOOTER_URL;
use crate::{Context, Error, HTTP_CLIENT, colors};

use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct CharacterObject {
    id: u32,
    char: String,
    simp_char: String,
    trad_char: String,
    freq: u32,
    stroke_count: u16,
    radical: String,
    simp_radical: String,
    heisig_keyword: Option<String>,
    heisig_number: Option<u32>,
    pinyin: String,
    pinyin_normalised: String,
    translations: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HanziResult {
    character_object: CharacterObject,
    is_radical: bool,
}

#[poise::command(prefix_command, slash_command, category = "Utility")]
pub async fn hanzi(
    ctx: Context<'_>,
    #[description = "Chinese character to lookup"] char: String,
) -> Result<(), Error> {
    let url = format!("https://api.hanzibase.net/character/{}",
        urlencoding::encode(&char));
    log::info!("Sending Request to {}", url);
    let resp = HTTP_CLIENT.get().unwrap().get(url).send().await?;

    let text = resp.text().await?;
    log::info!("Hanzi returned {}", text);

    let result: HanziResult = serde_json::from_str(text.as_str())?;
    let ch = &result.character_object;

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let mut fields = vec![
        ("Pinyin", ch.pinyin.clone(), true),
        ("Stroke Count", ch.stroke_count.to_string(), true),
        ("Frequency Rank", ch.freq.to_string(), true),
        ("Radical", ch.radical.clone(), true),
        ("Simplified", ch.simp_char.clone(), true),
        ("Traditional", ch.trad_char.clone(), true),
    ];

    if let Some(keyword) = &ch.heisig_keyword {
        fields.push(("Heisig Keyword", keyword.clone(), true));
    }

    if let Some(number) = ch.heisig_number {
        fields.push(("Heisig Number", number.to_string(), true));
    }

    fields.push((
        "Stroke Order",
        format!("[Link](https://www.strokeorder.com/chinese/{})", ch.char),
        false,
    ));

    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title(&ch.char)
            .description(&ch.translations.clone().unwrap_or("No translations available".to_string()))
            .fields(fields)
            .footer(footer)
            .color(colors::LAVENDER)
            .timestamp(serenity::model::Timestamp::now());

        poise::CreateReply::default().embed(embed)
    };

    ctx.send(reply).await?;
    Ok(())
}
