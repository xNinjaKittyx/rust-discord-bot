use crate::colors;
use crate::{Context, Error, HTTP_CLIENT};
use poise::serenity_prelude as serenity;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TvSearchResponse {
    results: Vec<TvResult>,
}

#[derive(Debug, Deserialize)]
struct TvResult {
    id: u64,
    name: String,
    original_name: String,
    overview: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    first_air_date: Option<String>,
    vote_average: f64,
    vote_count: u64,
    popularity: f64,
    origin_country: Vec<String>,
}

#[poise::command(
    prefix_command,
    slash_command,
    aliases("shows", "show"),
    category = "Entertainment"
)]
pub async fn tv(
    ctx: Context<'_>,
    #[description = "TV show title to search for"] query: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let url = format!(
        "https://api.themoviedb.org/3/search/tv?query={}",
        urlencoding::encode(&query)
    );

    let response = HTTP_CLIENT
        .get()
        .unwrap()
        .get(&url)
        .header(
            "Authorization",
            format!("Bearer {}", &*crate::env::TMDB_API_KEY),
        )
        .send()
        .await?;

    if !response.status().is_success() {
        ctx.say(format!("❌ API request failed: {}", response.status()))
            .await?;
        return Ok(());
    }

    let search_response: TvSearchResponse = response.json().await?;

    if search_response.results.is_empty() {
        ctx.say(format!("❌ No TV shows found for \"{}\"", query))
            .await?;
        return Ok(());
    }

    let show = &search_response.results[0];

    let mut embed = serenity::CreateEmbed::new()
        .title(&show.name)
        .url(format!("https://www.themoviedb.org/tv/{}", show.id))
        .color(colors::ACCENT);

    if let Some(ref overview) = show.overview {
        embed = embed.description(overview);
    }

    if let Some(ref poster) = show.poster_path {
        embed = embed.image(format!("https://image.tmdb.org/t/p/w500{}", poster));
    }

    if let Some(ref first_air) = show.first_air_date {
        embed = embed.field("First Aired", first_air, true);
    }

    embed = embed.field("Rating", format!("{:.1}/10", show.vote_average), true);
    embed = embed.field("Votes", show.vote_count.to_string(), true);

    if !show.origin_country.is_empty() {
        embed = embed.field("Origin", show.origin_country.join(", "), true);
    }

    if show.name != show.original_name {
        embed = embed.field("Original Name", &show.original_name, false);
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}
