use actix_web::{HttpResponse, Responder, get};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ChannelInfo {
    pub id: String,
    pub name: String,
    pub guild_id: String,
    pub guild_name: String,
}

#[derive(Deserialize, Serialize)]
pub struct CategoryInfo {
    pub id: String,
    pub name: String,
    pub guild_id: String,
    pub guild_name: String,
}

// Helper function to get Discord HTTP client
fn get_discord_http() -> Option<std::sync::Arc<poise::serenity_prelude::Http>> {
    crate::DISCORD_HTTP.get().cloned()
}

#[get("/api/channels")]
pub async fn get_channels() -> impl Responder {
    let http_option = get_discord_http();
    if http_option.is_none() {
        return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "Discord client not ready"
        }));
    }

    let http = http_option.unwrap();
    let mut channels = Vec::new();
    let mut categories = Vec::new();

    // Get guilds from HTTP
    match http.get_guilds(None, None).await {
        Ok(guilds) => {
            for guild_info in guilds {
                // Get channels for this guild
                match http.get_channels(guild_info.id).await {
                    Ok(guild_channels) => {
                        for channel in guild_channels {
                            match channel.kind {
                                poise::serenity_prelude::ChannelType::Text => {
                                    channels.push(ChannelInfo {
                                        id: channel.id.to_string(),
                                        name: channel.name.clone(),
                                        guild_id: guild_info.id.to_string(),
                                        guild_name: guild_info.name.clone(),
                                    });
                                }
                                poise::serenity_prelude::ChannelType::Category => {
                                    categories.push(CategoryInfo {
                                        id: channel.id.to_string(),
                                        name: channel.name.clone(),
                                        guild_id: guild_info.id.to_string(),
                                        guild_name: guild_info.name.clone(),
                                    });
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to get channels for guild {}: {:?}",
                            guild_info.id,
                            e
                        );
                    }
                }
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get guilds: {}", e)
            }));
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "channels": channels,
        "categories": categories
    }))
}
