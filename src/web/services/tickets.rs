use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TicketMenuRequest {
    pub id: String,
    pub channel_id: String,
    pub guild_id: String,
    pub category_id: String,
    pub embed_config: crate::tickets::EmbedConfig,
    pub button_config: crate::tickets::ButtonConfig,
    pub modal_config: crate::tickets::ModalConfig,
}

// Helper function to get Discord HTTP client
fn get_discord_http() -> Option<std::sync::Arc<poise::serenity_prelude::Http>> {
    crate::DISCORD_HTTP.get().cloned()
}

#[get("/api/tickets")]
pub async fn get_tickets() -> impl Responder {
    match crate::tickets::list_ticket_menus() {
        Ok(menus) => HttpResponse::Ok().json(serde_json::json!({
            "tickets": menus,
            "count": menus.len()
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to list ticket menus: {}", e)
        })),
    }
}

#[get("/api/tickets/{id}")]
pub async fn get_ticket(id: web::Path<String>) -> impl Responder {
    match crate::tickets::load_ticket_menu(&id) {
        Ok(Some(menu)) => HttpResponse::Ok().json(menu),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Ticket menu not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to load ticket menu: {}", e)
        })),
    }
}

#[post("/api/tickets")]
pub async fn create_ticket(ticket: web::Json<TicketMenuRequest>) -> impl Responder {
    let mut menu = crate::tickets::TicketMenu {
        id: ticket.id.clone(),
        channel_id: ticket.channel_id.clone(),
        message_id: None,
        guild_id: ticket.guild_id.clone(),
        category_id: ticket.category_id.clone(),
        embed_config: ticket.embed_config.clone(),
        button_config: ticket.button_config.clone(),
        modal_config: ticket.modal_config.clone(),
    };

    // Save to database
    match crate::tickets::save_ticket_menu(&menu) {
        Ok(_) => {
            // Post message to Discord
            if let Some(http) = get_discord_http() {
                if let Err(e) = menu.post_or_update_with_http(&http).await {
                    log::error!("Failed to post ticket menu message: {:?}", e);
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to post message to Discord: {}", e)
                    }));
                }
                // Save again with message_id
                if let Err(e) = crate::tickets::save_ticket_menu(&menu) {
                    log::error!("Failed to save ticket menu with message_id: {:?}", e);
                }
            }

            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "ticket": menu
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create ticket menu: {}", e)
        })),
    }
}

#[put("/api/tickets/{id}")]
pub async fn update_ticket(
    id: web::Path<String>,
    ticket: web::Json<TicketMenuRequest>,
) -> impl Responder {
    // Load existing menu to get message_id
    let existing_menu = match crate::tickets::load_ticket_menu(&id) {
        Ok(Some(menu)) => menu,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Ticket menu not found"
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to load ticket menu: {}", e)
            }));
        }
    };

    let mut menu = crate::tickets::TicketMenu {
        id: ticket.id.clone(),
        channel_id: ticket.channel_id.clone(),
        message_id: existing_menu.message_id,
        guild_id: ticket.guild_id.clone(),
        category_id: ticket.category_id.clone(),
        embed_config: ticket.embed_config.clone(),
        button_config: ticket.button_config.clone(),
        modal_config: ticket.modal_config.clone(),
    };

    // Save to database
    match crate::tickets::save_ticket_menu(&menu) {
        Ok(_) => {
            // Update message in Discord
            if let Some(http) = get_discord_http() {
                if let Err(e) = menu.post_or_update_with_http(&http).await {
                    log::error!("Failed to update ticket menu message: {:?}", e);
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to update message in Discord: {}", e)
                    }));
                }
                // Save again in case message_id changed
                if let Err(e) = crate::tickets::save_ticket_menu(&menu) {
                    log::error!("Failed to save ticket menu after update: {:?}", e);
                }
            }

            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "ticket": menu
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update ticket menu: {}", e)
        })),
    }
}

#[delete("/api/tickets/{id}")]
pub async fn delete_ticket(id: web::Path<String>) -> impl Responder {
    // Load the ticket menu first to get message info
    let menu = crate::tickets::load_ticket_menu(&id);

    // Try to delete the Discord message if it exists
    if let Ok(Some(ticket_menu)) = menu
        && let (Some(message_id), Some(http)) =
            (ticket_menu.message_id.as_ref(), get_discord_http())
        && let Ok(channel_id_u64) = ticket_menu.channel_id.parse::<u64>()
        && let Ok(message_id_u64) = message_id.parse::<u64>()
    {
        let channel_id = poise::serenity_prelude::ChannelId::new(channel_id_u64);
        let msg_id = poise::serenity_prelude::MessageId::new(message_id_u64);

        // Try to delete, but don't fail if it doesn't work
        if let Err(e) = http.delete_message(channel_id, msg_id, None).await {
            log::warn!("Failed to delete ticket menu message: {:?}", e);
        }
    }

    // Delete from database
    match crate::tickets::delete_ticket_menu(&id) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "id": id.as_str()
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to delete ticket menu: {}", e)
        })),
    }
}
