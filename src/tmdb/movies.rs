use crate::colors;
use crate::{Context, Error, HTTP_CLIENT};
use poise::serenity_prelude as serenity;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MovieSearchResponse {
    results: Vec<MovieResult>,
}

#[derive(Debug, Deserialize)]
struct MovieResult {
    id: u64,
    title: String,
    original_title: String,
    overview: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    release_date: Option<String>,
    vote_average: f64,
    vote_count: u64,
    popularity: f64,
}

#[poise::command(
    prefix_command,
    slash_command,
    aliases("movies"),
    category = "Entertainment"
)]
pub async fn movie(
    ctx: Context<'_>,
    #[description = "Movie title to search for"] query: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let url = format!(
        "https://api.themoviedb.org/3/search/movie?query={}",
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

    let search_response: MovieSearchResponse = response.json().await?;

    if search_response.results.is_empty() {
        ctx.say(format!("❌ No movies found for \"{}\"", query))
            .await?;
        return Ok(());
    }

    let movie = &search_response.results[0];

    let mut embed = serenity::CreateEmbed::new()
        .title(&movie.title)
        .url(format!("https://www.themoviedb.org/movie/{}", movie.id))
        .color(colors::ACCENT);

    if let Some(ref overview) = movie.overview {
        embed = embed.description(overview);
    }

    if let Some(ref poster) = movie.poster_path {
        embed = embed.image(format!("https://image.tmdb.org/t/p/w500{}", poster));
    }

    if let Some(ref release) = movie.release_date {
        embed = embed.field("Release Date", release, true);
    }

    embed = embed.field("Rating", format!("{:.1}/10", movie.vote_average), true);
    embed = embed.field("Votes", movie.vote_count.to_string(), true);

    if movie.title != movie.original_title {
        embed = embed.field("Original Title", &movie.original_title, false);
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}
