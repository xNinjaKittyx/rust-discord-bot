use std::collections::{HashMap, HashSet};
use std::fs;
use std::time::{Duration, SystemTime};

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
    i_d: u32,
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
struct IDWithoutMAL {
    ani_d_b: u32,
    i_d: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ReleaseGroup {
    i_d: u32,
    name: Option<String>,
    short_name: Option<String>,
    source: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AniDBEpisode {
    release_group: ReleaseGroup,
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
struct Episode {
    i_ds: IDWithoutMAL,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MediaInfo {
    bit_rate: u32,
    video: Vec<Video>,
    audio: Vec<Audio>,
    subtitles: Vec<Subtitle>,
    chapters: Vec<Chapter>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Video {
    resolution: String,
    bit_rate: u32,
    bit_depth: u8,
    codec: Codec,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Audio {
    channels: u8,
    channel_layout: Option<String>,
    samples_per_frame: u32,
    sampling_rate: u32,
    compression_mode: String,
    bit_rate: u32,
    codec: Codec,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Subtitle {
    title: Option<String>,
    order: u32,
    language: String,
    codec: Codec,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Chapter {
    title: String,
    language: String,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Codec {
    simplified: String,
    raw: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct File {
    i_d: u32,
    size: u64,
    ani_d_b: Option<AniDBEpisode>,
    media_info: Option<MediaInfo>,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FileSource {
    unknown: u32,
    other: u32,
    t_v: u32,
    d_v_d: u32,
    blu_ray: u32,
    web: u32,
    v_h_s: u32,
    v_c_d: u32,
    laser_disc: u32,
    camera: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Sizes {
    file_sources: FileSource
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
    sizes: Sizes,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ShokoFileResponse {
    total: u32,
    list: Vec<File>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ShokoEpisodeResponse {
    total: u32,
    list: Vec<Episode>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ShokoSeriesResponse {
    list: Vec<Anime>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ShokoStats {
    file_count: u32,
    series_count: u32,
    group_count: u32,
    file_size: u64,
    finished_series: u32,
    watched_episodes: u32,
    watched_hours: f64,
    percent_duplicate: f64,
    missing_episodes: u32,
    missing_episodes_collecting: u32,
    unrecognized_files: u32,
    series_with_missing_links: u32,
    episodes_with_multiple_files: u32,
    files_with_duplicate_locations: u32,
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

fn format_bytes(bytes: u64) -> String {
    const TB: u64 = 1_099_511_627_776; // 1024^4
    const GB: u64 = 1_073_741_824;     // 1024^3
    const MB: u64 = 1_048_576;         // 1024^2
    const KB: u64 = 1_024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

async fn get_anilist_id_from_mal_id(
    mal_id: u32,
) -> Result<Option<u32>, Box<dyn std::error::Error>> {
    let url = "https://raw.githubusercontent.com/Kometa-Team/Anime-IDs/master/anime_ids.json";
    let local_path = "anime_ids.json";
    let update_interval = Duration::from_secs(60 * 60 * 24); // 24 hours

    let needs_update = match fs::metadata(local_path) {
        Ok(meta) => match meta.modified() {
            Ok(modified) => {
                SystemTime::now()
                    .duration_since(modified)
                    .unwrap_or(update_interval + Duration::from_secs(1))
                    > update_interval
            }
            Err(_) => true,
        },
        Err(_) => true,
    };

    if needs_update {
        let resp = reqwest::get(url).await?;
        let remote_bytes = resp.bytes().await?;
        fs::write(local_path, &remote_bytes)?;
    }

    let data = fs::read(local_path)?;
    let json: HashMap<String, serde_json::Value> = serde_json::from_slice(&data)?;
    for (_key, value) in json.iter() {
        if let Some(local_mal_id) = value.get("mal_id").and_then(|v| v.as_u64()) {
            if local_mal_id == mal_id as u64 {
                if let Some(anilist_id) = value.get("anilist_id").and_then(|v| v.as_u64()) {
                    return Ok(Some(anilist_id as u32));
                }
            }
        }
    }
    Ok(None)
}

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("search", "stats"),
    subcommand_required,
    category = "Anime"
)]
pub async fn shoko(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Anime")]
pub async fn stats(ctx: Context<'_>) -> Result<(), Error> {
    let resp = HTTP_CLIENT
        .get()
        .unwrap()
        .get(format!("{}/api/v3/Dashboard/Stats", &*SHOKO_SERVER_URL))
        .header("apikey", &*SHOKO_SERVER_API_KEY)
        .send()
        .await?;

    let text = resp.text().await?;
    log::info!("Shoko Server returned {}", text.as_str());

    let stats: ShokoStats = serde_json::from_str(text.as_str())?;

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title("VFS Anime Stats")
            .fields(vec![
                ("Files Count", format!("{}", &stats.file_count), true),
                ("Series Count", format!("{}", &stats.series_count), true),
                ("File Size", format_bytes(stats.file_size), true),
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

#[poise::command(prefix_command, slash_command, category = "Anime")]
pub async fn search(ctx: Context<'_>, value: String) -> Result<(), Error> {
    let map = ShokoSeriesRequest {
        apply_at_series_level: false,
        expression: Expression {
            left: ExpressionType {
                r#type: "NamesSelector".to_string(),
            },
            parameter: value,
            r#type: "AnyContains".to_string(),
        },
    };
    let result;
    log::info!("Making HTTP Request to Shoko Server with AnyContains");
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

    let text = resp.text().await?;
    log::info!("Shoko Server returned {}", text.as_str());
    let results: ShokoSeriesResponse = serde_json::from_str(text.as_str())?;

    if !results.list.is_empty() {
        result = find_best_scoring_string(&map.expression.parameter, &results);
    } else {
        log::warn!("No results found for search: {}", map.expression.parameter);
        ctx.send(poise::CreateReply::default().content("No results found."))
            .await?;
        return Ok(());
    }

    log::info!("Fetching poster, series details, and episodes in parallel");

    // Start all three API calls in parallel
    let poster_future = async {
        HTTP_CLIENT
            .get()
            .unwrap()
            .get(format!(
                "{}/api/v3/Image/AniDB/Poster/{}",
                &*SHOKO_SERVER_URL, &result.images.posters[0].i_d
            ))
            .send()
            .await
    };

    let series_future = async {
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
        log::info!("Shoko Server returned series data");
        let series: Anime = serde_json::from_str(text.as_str())?;
        Ok::<Anime, Error>(series)
    };

    let episodes_future = async {
        let resp = HTTP_CLIENT
            .get()
            .unwrap()
            .get(format!(
                "{}/api/v3/Series/{}/Episode?pageSize=1&includeMissing=false&includeWatched=true&includeHidden=false&includeUnaired=false",
                &*SHOKO_SERVER_URL, &result.i_ds.i_d
            ))
            .header("apikey", &*SHOKO_SERVER_API_KEY)
            .send()
            .await?;
        let text = resp.text().await?;
        log::info!("Shoko Server returned episode data");
        let episodes: ShokoEpisodeResponse = serde_json::from_str(text.as_str())?;
        Ok::<ShokoEpisodeResponse, Error>(episodes)
    };

    // Await poster first to send initial embed quickly
    let poster = poster_future.await?;
    let attachment: serenity::CreateAttachment =
        serenity::CreateAttachment::bytes(poster.bytes().await.unwrap(), "poster.png");

    // Send initial embed with basic info
    let initial_embed = serenity::CreateEmbed::new()
        .title(&result.name)
        .description("Loading anime details...")
        .image("attachment://poster.png")
        .color(0x5dadec);

    let initial_reply = poise::CreateReply::default()
        .embed(initial_embed)
        .attachment(attachment.clone());
    let reply_handle = ctx.send(initial_reply).await?;

    // Now await the remaining API calls
    let (series_result, episodes_result) = tokio::join!(series_future, episodes_future);
    let series: Anime = series_result?;
    let episodes: ShokoEpisodeResponse = episodes_result?;

    let anidb = &series.ani_d_b.unwrap();

    let mut mal_vec: Vec<String> = Vec::new();
    let mut mal_id: Option<u32> = None;
    for value in &result.i_ds.m_a_l {
        mal_vec.push(format!(
            "[MyAnimeList](https://myanimelist.net/anime/{})",
            value
        ));

        // Some have multiple mal IDs, generally the first one is the right one.
        if mal_id.is_none() {
            mal_id = Some(*value);
        }
    }
    let mal_id = mal_id.unwrap_or(0);
    let mal_string = mal_vec.join("\n");

    let mut release_group: String = "Unknown Release Group".to_string();
    let mut video_codec: String = "Unknown Video Codec".to_string();
    let mut audio_codec: String = "Unknown Audio Codec".to_string();
    let mut video_bit_depth: u8 = 10;
    let mut video_resolution: String = "Unknown Resolution".to_string();
    let mut video_bit_rate: f32 = 0.0;

    if episodes.total == 0 {
        log::warn!("No episodes found for series {}", result.name);
    } else {
        // Grab the File so we can figure out the release group.
        let resp = HTTP_CLIENT
            .get()
            .unwrap()
            .get(format!(
                "{}/api/v3/Episode/{}/File?includeDataFrom=AniDB&include=MediaInfo",
                &*SHOKO_SERVER_URL, &episodes.list[0].i_ds.i_d
            ))
            .header("apikey", &*SHOKO_SERVER_API_KEY)
            .send()
            .await?;
        let text = resp.text().await?;
        log::info!("Shoko Server returned {}", text.as_str());
        let files: ShokoFileResponse = serde_json::from_str(text.as_str())?;

        if let Some(file) = files.list.first() {
            let media_info = file.media_info.as_ref().unwrap();
            video_codec = media_info.video.first().unwrap().codec.simplified.clone();
            audio_codec = media_info.audio.first().unwrap().codec.simplified.clone();
            video_bit_depth = media_info.video.first().unwrap().bit_depth.clone();
            video_resolution = media_info.video.first().unwrap().resolution.clone();
            video_bit_rate = media_info.bit_rate as f32 / 1024.0 / 1024.0; // Convert to Mb/s

            if let Some(ani_d_b) = &file.ani_d_b {
                release_group = ani_d_b
                    .release_group
                    .name
                    .clone()
                    .unwrap_or(release_group);
            } else {
                log::warn!("No AniDB data found for file ID {}", file.i_d);
            }
        } else {
            log::warn!("No files found for series {}", result.name);
            release_group = "No Files Found".to_string();
        }
    }

    // Info from SeaDex if this release is a good one.

    let alid: u32 = get_anilist_id_from_mal_id(mal_id)
        .await
        .unwrap_or(None)
        .unwrap_or(0);
    log::info!("mal_id {} - alid {}", mal_id, alid);

    // Convert AniDB ID to AniListID
    let mut group_list = "No releases found".to_string();
    if alid != 0 {
        let resp = HTTP_CLIENT
            .get()
            .unwrap()
            .get(format!(
                "https://releases.moe/api/collections/entries/records?filter=alID={}&expand=trs&filter=trs.tracker=%27Nyaa%27&fields=expand.trs.releaseGroup",
                &alid,
            ))
            .send()
            .await?;
        let text = resp.text().await?;
        log::info!("Releases.moe returned {}", text.as_str());

        // Extract unique release group names
        let mut release_groups = HashSet::new();
        let json: serde_json::Value = serde_json::from_str(&text)?;
        if let Some(items) = json.get("items").and_then(|v| v.as_array()) {
            for item in items {
                if let Some(trs) = item
                    .get("expand")
                    .and_then(|expand| expand.get("trs"))
                    .and_then(|trs| trs.as_array())
                {
                    for tr in trs {
                        if let Some(group) = tr.get("releaseGroup").and_then(|g| g.as_str()) {
                            release_groups.insert(group.to_string());
                        }
                    }
                }
            }
        }
        // Join the set into a comma-separated string
        group_list = release_groups.into_iter().collect::<Vec<_>>().join(", ");
        if group_list.is_empty() {
            group_list = "No Good Release Available".to_string();
        }
    }

    // Collect file sources with count > 0
    let mut file_sources = Vec::new();
    let fs = &series.sizes.file_sources;

    if fs.unknown > 0 { file_sources.push("Unknown"); }
    if fs.other > 0 { file_sources.push("Other"); }
    if fs.t_v > 0 { file_sources.push("TV"); }
    if fs.d_v_d > 0 { file_sources.push("DVD"); }
    if fs.blu_ray > 0 { file_sources.push("BD"); }
    if fs.web > 0 { file_sources.push("Web"); }
    if fs.v_h_s > 0 { file_sources.push("VHS"); }
    if fs.v_c_d > 0 { file_sources.push("VCD"); }
    if fs.laser_disc > 0 { file_sources.push("LaserDisc"); }
    if fs.camera > 0 { file_sources.push("Camera"); }

    let file_sources_text = if file_sources.is_empty() {
        "None".to_string()
    } else {
        file_sources.join(" | ")
    };

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title(&result.name)
            .url(&result.links[0].u_r_l)
            .description(format_links(&anidb.description))
            .image("attachment://poster.png")
            .fields(vec![
                ("Episodes", format!("{}", &anidb.episode_count), true),
                ("Aired Date", format!("{}", &anidb.air_date), true),
                (
                    "Links",
                    format!("[AniDB](https://anidb.net/anime/{})\n{}\n[AniList](https://anilist.co/anime/{})", result.i_ds.ani_d_b, mal_string, alid),
                    true,
                ),
                (
                    "Releases.moe",
                    format!("[{}](https://releases.moe/{})", group_list, alid),
                    true,
                ),
                (
                    "Current Shoko Release",
                    format!("[{}]({}/webui/collection/series/{}/overview)", release_group, &*SHOKO_SERVER_URL, result.i_ds.i_d),
                    true,
                ),
                (
                    "File Details",
                    format!("[{}][{}][{}-bit][{}][{:.2} Mb/s]", video_codec, audio_codec, video_bit_depth, video_resolution, video_bit_rate),
                    true,
                ),
                (
                    "File Sources",
                    file_sources_text,
                    false,
                )
            ])
            .footer(footer)
            // Add a timestamp for the current time
            // This also accepts a rfc3339 Timestamp
            .timestamp(serenity::model::Timestamp::now());

        poise::CreateReply::default()
            .embed(embed)
    };

    reply_handle.edit(ctx, reply).await?;
    Ok(())
}
