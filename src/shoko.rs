use crate::env::{FOOTER_URL, SHOKO_SERVER_API_KEY, SHOKO_SERVER_URL};
use crate::{Context, Error, HTTP_CLIENT};

use poise::serenity_prelude as serenity;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use strsim::jaro_winkler;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ExpressionType {
    r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Expression {
    left: ExpressionType,
    parameter: String,
    r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ShokoSeriesRequest {
    apply_at_series_level: bool,
    expression: Expression,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Link {
    r#type: String,
    name: String,
    u_r_l: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Poster {
    i_d: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Image {
    posters: Vec<Poster>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct IDs {
    ani_d_b: u32,
    m_a_l: Vec<u32>,
    i_d: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AniDB {
    description: String,
    air_date: String,
    end_date: String,
    episode_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Anime {
    name: String,
    images: Image,
    i_ds: IDs,
    size: u32,
    links: Vec<Link>,
    ani_d_b: Option<AniDB>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ShokoSeriesResponse {
    list: Vec<Anime>,
}

fn format_links(text: &str) -> String {
    let pattern = r"(http://anidb.net/.*?) \[([^\]]+)\]"; // Regex pattern
    let re = Regex::new(pattern).unwrap();
    re.replace_all(text, |m: &Captures| {
        format!(
            "[{}]({})",
            m.get(2).unwrap().as_str(),
            m.get(1).unwrap().as_str()
        )
    })
    .to_string()
}

fn find_best_scoring_string<'a>(text: &String, results: &'a ShokoSeriesResponse) -> &'a Anime {
    let mut best_score = 0.0;
    let mut best_match: &Anime = &results.list[0];

    for anime in &results.list {
        let score = jaro_winkler(&text, anime.name.as_str()) as f64;
        if score > best_score {
            best_score = score;
            best_match = anime;
        }
    }

    best_match
}

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("search"),
    subcommand_required
)]
pub async fn shoko(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn search(ctx: Context<'_>, value: String) -> Result<(), Error> {
    let mut map = ShokoSeriesRequest {
        apply_at_series_level: false,
        expression: Expression {
            left: ExpressionType {
                r#type: "NameSelector".to_string(),
            },
            parameter: value,
            r#type: "StringEquals".to_string(),
        },
    };
    log::info!("Making HTTP Search Request to Shoko Server with Exact Match");
    let resp = HTTP_CLIENT
        .get()
        .unwrap()
        .post(format!(
            "{}/api/v3/Filter/Preview/Series?pageSize=1&page=1",
            &*SHOKO_SERVER_URL
        ))
        .header("apikey", &*SHOKO_SERVER_API_KEY)
        .json(&map)
        .send()
        .await?;

    let mut text = resp.text().await?;
    log::info!("Shoko Server returned {}", text.as_str());
    let mut results: ShokoSeriesResponse = serde_json::from_str(text.as_str())?;
    let result;
    if results.list.is_empty() {
        log::info!("Making HTTP Request to Shoko Server with Fuzzy Match");
        map.expression.r#type = "StringFuzzyMatches".to_string();
        let resp = HTTP_CLIENT
            .get()
            .unwrap()
            .post(format!(
                "{}/api/v3/Filter/Preview/Series?pageSize=100&page=1",
                &*SHOKO_SERVER_URL
            ))
            .header("apikey", &*SHOKO_SERVER_API_KEY)
            .json(&map)
            .send()
            .await?;

        text = resp.text().await?;
        log::info!("Shoko Server returned {}", text.as_str());
        results = serde_json::from_str(text.as_str())?;

        result = find_best_scoring_string(&map.expression.parameter, &results);
    } else {
        result = &results.list[0];
    }

    let mut mal_vec: Vec<String> = Vec::new();
    for value in &result.i_ds.m_a_l {
        mal_vec.push(format!("[Link](https://myanimelist.net/anime/{})", value))
    }
    let mal_string = mal_vec.join("\n");

    log::info!("GET Shoko Server Image Poster Data");

    let poster = HTTP_CLIENT
        .get()
        .unwrap()
        .get(format!(
            "{}/api/v3/Image/AniDB/Poster/{}",
            &*SHOKO_SERVER_URL, &result.images.posters[0].i_d
        ))
        .send()
        .await?;

    let attachment: serenity::CreateAttachment =
        serenity::CreateAttachment::bytes(poster.bytes().await.unwrap(), "poster.png");

    log::info!("GET Shoko Server Detailed Data");

    let resp = HTTP_CLIENT
        .get()
        .unwrap()
        .get(format!(
            "{}/api/v3/Series/{}?includeDataFrom=AniDB",
            &*SHOKO_SERVER_URL, &result.i_ds.i_d
        ))
        .header("apikey", &*SHOKO_SERVER_API_KEY)
        .send()
        .await?;
    let text = resp.text().await?;
    log::trace!("Shoko Server returned {}", text.as_str());

    let series: Anime = serde_json::from_str(text.as_str())?;

    let anidb = &series.ani_d_b.unwrap();

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title(&result.name)
            .description(format_links(&anidb.description))
            .image("attachment://poster.png")
            .fields(vec![
                ("Episodes", format!("{}", &anidb.episode_count), true),
                ("Aired Date", format!("{}", &anidb.air_date), true),
                (
                    "AniDB",
                    format!("[Link](https://anidb.net/anime/{})", result.i_ds.ani_d_b),
                    false,
                ),
                ("MyAnimeList", mal_string, false),
                (
                    "Source",
                    format!("[Link]({})", result.links[0].u_r_l),
                    false,
                ),
            ])
            .footer(footer)
            // Add a timestamp for the current time
            // This also accepts a rfc3339 Timestamp
            .timestamp(serenity::model::Timestamp::now());

        poise::CreateReply::default()
            .embed(embed)
            .attachment(attachment)
    };

    ctx.send(reply).await?;
    Ok(())
}
