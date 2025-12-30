use crate::{Context, Error, HTTP_CLIENT, KV_DATABASE, STREAMS};
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

#[derive(Debug, Serialize, Deserialize)]
struct StreamFollow {
    user_id: u64,
    url: String,
    channel_id: u64,
    platform: String,
    channel_name: String,
}

#[derive(Debug, Deserialize)]
struct KickOAuthResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct KickApiResponse {
    data: Vec<KickChannelData>,
}

#[derive(Debug, Deserialize)]
struct KickChannelData {
    broadcaster_user_id: u64,
    slug: String,
    channel_description: Option<String>,
    banner_picture: Option<String>,
    stream: Option<KickStream>,
    stream_title: Option<String>,
    category: Option<KickCategory>,
}

#[derive(Debug, Deserialize)]
struct KickStream {
    is_live: bool,
    is_mature: Option<bool>,
    language: Option<String>,
    start_time: Option<String>,
    viewer_count: u32,
    thumbnail: Option<String>,
}

#[derive(Debug, Deserialize)]
struct KickCategory {
    id: u64,
    name: String,
    thumbnail: Option<String>,
}

// Track which streams are currently live with their message IDs and start times
// Format: HashMap<stream_key, (message_id, start_time)>
static LIVE_STREAMS: LazyLock<Arc<Mutex<HashMap<String, (u64, String)>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

fn parse_kick_url(url: &str) -> Option<String> {
    // Parse kick.com URLs like https://kick.com/channelname
    if url.contains("kick.com/") {
        if let Some(channel) = url.split("kick.com/").nth(1) {
            let channel_name = channel.trim_end_matches('/').to_string();
            if !channel_name.is_empty() {
                return Some(channel_name);
            }
        }
    }
    None
}

#[poise::command(prefix_command, slash_command, check = "crate::permissions::check_trusted", category = "Streams")]
pub async fn follow(
    ctx: Context<'_>,
    #[description = "Stream URL to follow"] url: String,
) -> Result<(), Error> {
    // Parse the URL to determine platform and channel
    let (platform, channel_name) = if let Some(channel) = parse_kick_url(&url) {
        ("kick", channel)
    } else {
        ctx.say("❌ Unsupported URL. Currently only kick.com URLs are supported.").await?;
        return Ok(());
    };

    let user_id = ctx.author().id.get();
    let channel_id = ctx.channel_id().get();

    let follow = StreamFollow {
        user_id,
        url: url.clone(),
        channel_id,
        platform: platform.to_string(),
        channel_name: channel_name.clone(),
    };

    // Store in database
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(STREAMS)?;
        let key = format!("{}:{}", platform, channel_name);
        let value = serde_json::to_string(&follow)?;
        table.insert(key.as_str(), value.as_str())?;
    }
    tx.commit()?;

    ctx.say(format!(
        "✅ Now following **{}** on {}. Notifications will be posted in <#{}>",
        channel_name, platform, channel_id
    ))
    .await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, check = "crate::permissions::check_trusted", category = "Streams")]
pub async fn unfollow(
    ctx: Context<'_>,
    #[description = "Stream URL to unfollow"] url: String,
) -> Result<(), Error> {
    let (platform, channel_name) = if let Some(channel) = parse_kick_url(&url) {
        ("kick", channel)
    } else {
        ctx.say("❌ Unsupported URL. Currently only kick.com URLs are supported.").await?;
        return Ok(());
    };

    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(STREAMS)?;
        let key = format!("{}:{}", platform, channel_name);
        table.remove(key.as_str())?;
    }
    tx.commit()?;

    ctx.say(format!("✅ Unfollowed **{}** on {}", channel_name, platform))
        .await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Streams")]
pub async fn following(ctx: Context<'_>) -> Result<(), Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_read()?;
    let table = tx.open_table(STREAMS)?;

    let mut follows: Vec<StreamFollow> = Vec::new();
    for item in table.range::<&str>(..)? {
        let (_, value) = item?;
        if let Ok(follow) = serde_json::from_str::<StreamFollow>(value.value()) {
            follows.push(follow);
        }
    }

    if follows.is_empty() {
        ctx.say("No streams are currently being followed.").await?;
        return Ok(());
    }

    let mut response = String::from("**Following streams:**\n");
    for follow in follows {
        response.push_str(&format!(
            "• **{}** on {} - Notifications in <#{}> (followed by <@{}>)\n",
            follow.channel_name, follow.platform, follow.channel_id, follow.user_id
        ));
    }

    ctx.say(response).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Streams")]
pub async fn preview(
    ctx: Context<'_>,
    #[description = "Kick channel name to preview"] channel_name: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    match check_kick_stream(&channel_name).await {
        Ok(Some(channel)) => {
            if let Some(ref stream) = channel.stream {
                if stream.is_live {
                    let stream_url = format!("https://kick.com/{}", channel_name);
                    let embed = create_stream_embed(&channel, &stream_url, stream);

                    ctx.send(
                        poise::CreateReply::default()
                            .embed(embed),
                    )
                    .await?;
                } else {
                    ctx.say(format!("❌ **{}** is not currently live on Kick.", channel_name))
                        .await?;
                }
            } else {
                ctx.say(format!("❌ **{}** is not currently live on Kick.", channel_name))
                    .await?;
            }
        }
        Ok(None) => {
            ctx.say(format!("❌ Channel **{}** not found on Kick.", channel_name))
                .await?;
        }
        Err(e) => {
            ctx.say(format!("❌ Error checking stream: {}", e)).await?;
        }
    }

    Ok(())
}

async fn get_kick_oauth_token() -> Result<String, Error> {
    let client = HTTP_CLIENT.get().unwrap();

    let params = [
        ("grant_type", "client_credentials"),
        ("client_id", &*crate::env::KICK_CLIENT_ID),
        ("client_secret", &*crate::env::KICK_CLIENT_SECRET),
    ];

    let response = client
        .post("https://id.kick.com/oauth/token")
        .form(&params)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("OAuth token request failed: {}", response.status()).into());
    }

    let oauth_response: KickOAuthResponse = response.json().await?;
    Ok(oauth_response.access_token)
}

async fn check_kick_stream(channel_name: &str) -> Result<Option<KickChannelData>, Error> {
    let access_token = get_kick_oauth_token().await?;
    let url = format!("https://api.kick.com/public/v1/channels?slug={}", channel_name);

    log::debug!("Checking Kick channel: {}", url);

    let response = HTTP_CLIENT
        .get()
        .unwrap()
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;

    log::debug!("Received response with status: {}", response.status());

    if response.status().is_success() {
        let api_response: KickApiResponse = response.json().await?;
        Ok(api_response.data.into_iter().next())
    } else {
        Ok(None)
    }
}

fn create_stream_embed(
    channel: &KickChannelData,
    stream_url: &str,
    stream: &KickStream,
) -> serenity::CreateEmbed {
    let mut embed = serenity::CreateEmbed::new()
        .url(stream_url)
        .color(0x53fc18)
        .timestamp(serenity::Timestamp::now());

    // Set title from stream_title
    if let Some(ref stream_title) = channel.stream_title {
        embed = embed.title(stream_title);
    }

    // Set description starting with slug
    let mut description = format!("**{}**", channel.slug);
    if let Some(ref channel_desc) = channel.channel_description {
        if !channel_desc.is_empty() {
            description.push_str(&format!("\n{}", channel_desc));
        }
    }
    embed = embed.description(description);

    // Set image to thumbnail
    if let Some(ref thumbnail) = stream.thumbnail {
        embed = embed.image(thumbnail);
    }

    // Add viewer count field
    embed = embed.field("Viewers", stream.viewer_count.to_string(), true);

    // Add category field
    if let Some(ref category) = channel.category {
        embed = embed.field("Category", &category.name, true);
    }

    embed
}

async fn check_and_notify_streams(http: &serenity::Http) -> Result<(), Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_read()?;
    let table = tx.open_table(STREAMS)?;

    let mut follows: Vec<StreamFollow> = Vec::new();
    for item in table.range::<&str>(..)? {
        let (_, value) = item?;
        if let Ok(follow) = serde_json::from_str::<StreamFollow>(value.value()) {
            follows.push(follow);
        }
    }
    drop(table);
    drop(tx);

    for follow in follows {
        if follow.platform != "kick" {
            continue;
        }

        match check_kick_stream(&follow.channel_name).await {
            Ok(Some(channel)) => {
                let stream_key = format!("{}:{}", follow.platform, follow.channel_name);

                // Check if stream has stream data and is actually live
                let is_live = if let Some(ref stream) = channel.stream {
                    stream.is_live
                } else {
                    false
                };

                if is_live {
                    if let Some(ref stream) = channel.stream {
                        let mut live_streams = LIVE_STREAMS.lock().await;

                        let discord_channel = serenity::ChannelId::new(follow.channel_id);
                        let embed = create_stream_embed(&channel, &follow.url, stream);
                        let current_start_time = stream.start_time.clone().unwrap_or_default();

                        // Check if we already have a message for this stream
                        if let Some(&(message_id, ref stored_start_time)) = live_streams.get(&stream_key) {
                            // Compare start times to see if it's the same stream session
                            if current_start_time == *stored_start_time {
                                // Same stream session, edit the existing message
                                let message = discord_channel.message(http, message_id).await;
                                if let Ok(mut msg) = message {
                                    let edit = serenity::EditMessage::new()
                                        .embed(embed);
                                    if let Err(e) = msg.edit(http, edit).await {
                                        log::error!("Failed to edit stream notification: {}", e);
                                        // If edit fails, remove the entry and send a new one next time
                                        live_streams.remove(&stream_key);
                                    }
                                } else {
                                    // Message doesn't exist anymore, remove from tracking and send new one
                                    live_streams.remove(&stream_key);

                                    let message = serenity::CreateMessage::new()
                                        .embed(embed);

                                    if let Ok(sent_msg) = discord_channel.send_message(http, message).await {
                                        live_streams.insert(stream_key.clone(), (sent_msg.id.get(), current_start_time));
                                    } else {
                                        log::error!("Failed to send stream notification");
                                    }
                                }
                            } else {
                                // Different start_time means new stream session, send a new message
                                log::info!("New stream session detected for {}, creating new notification", follow.channel_name);
                                live_streams.remove(&stream_key);

                                let message = serenity::CreateMessage::new()
                                    .embed(embed);

                                if let Ok(sent_msg) = discord_channel.send_message(http, message).await {
                                    live_streams.insert(stream_key.clone(), (sent_msg.id.get(), current_start_time));
                                } else {
                                    log::error!("Failed to send stream notification");
                                }
                            }
                        } else {
                            // Send a new notification and store the message ID with start_time
                            let message = serenity::CreateMessage::new()
                                .embed(embed);

                            match discord_channel.send_message(http, message).await {
                                Ok(sent_msg) => {
                                    live_streams.insert(stream_key.clone(), (sent_msg.id.get(), current_start_time));
                                }
                                Err(e) => {
                                    log::error!("Failed to send stream notification: {}", e);
                                }
                            }
                        }
                    }
                } else {
                    // Stream is offline, remove from live set
                    let mut live_streams = LIVE_STREAMS.lock().await;
                    live_streams.remove(&stream_key);
                }
            }
            Ok(None) => {
                log::warn!("Channel {} not found on Kick", follow.channel_name);
            }
            Err(e) => {
                log::error!("Error checking stream {}: {}", follow.channel_name, e);
            }
        }
    }

    Ok(())
}

pub async fn start_stream_checker(http: Arc<serenity::Http>) {
    log::info!("Starting stream checker background task");

    let mut check_interval = interval(Duration::from_secs(60));

    loop {
        check_interval.tick().await;

        if let Err(e) = check_and_notify_streams(&http).await {
            log::error!("Error in stream checker: {}", e);
        }
    }
}
