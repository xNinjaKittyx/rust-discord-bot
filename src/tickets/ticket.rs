use crate::colors;
use crate::{Error, KV_DATABASE};
use poise::serenity_prelude as serenity;
use redb::ReadableDatabase;
use serde::{Deserialize, Serialize};

// Table definitions
const TICKETS: redb::TableDefinition<&str, &str> = redb::TableDefinition::new("tickets");
const ACTIVE_TICKETS: redb::TableDefinition<&str, &str> =
    redb::TableDefinition::new("active_tickets");

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TicketMenu {
    pub id: String,
    pub channel_id: String,
    pub message_id: Option<String>,
    pub guild_id: String,
    pub category_id: String,
    pub embed_config: EmbedConfig,
    pub button_config: ButtonConfig,
    pub modal_config: ModalConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbedConfig {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub color: Option<u32>,
    pub author_name: Option<String>,
    pub author_url: Option<String>,
    pub author_icon_url: Option<String>,
    pub footer_text: Option<String>,
    pub footer_icon_url: Option<String>,
    pub image_url: Option<String>,
    pub thumbnail_url: Option<String>,
}

impl Default for EmbedConfig {
    fn default() -> Self {
        Self {
            title: Some("🎫 Support Tickets".to_string()),
            description: Some("Click the button below to create a support ticket.".to_string()),
            url: None,
            color: Some(colors::PRIMARY),
            author_name: None,
            author_url: None,
            author_icon_url: None,
            footer_text: None,
            footer_icon_url: None,
            image_url: None,
            thumbnail_url: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ButtonConfig {
    pub label: String,
    pub emoji: Option<String>,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            label: "Create Ticket".to_string(),
            emoji: Some("🎫".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModalConfig {
    pub title_field: ModalField,
    pub description_field: ModalField,
}

impl Default for ModalConfig {
    fn default() -> Self {
        Self {
            title_field: ModalField {
                label: "Title".to_string(),
                placeholder: Some("Brief description of your issue".to_string()),
                style: TextInputStyle::Short,
            },
            description_field: ModalField {
                label: "Description".to_string(),
                placeholder: Some("Please provide details about your request".to_string()),
                style: TextInputStyle::Paragraph,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModalField {
    pub label: String,
    pub placeholder: Option<String>,
    pub style: TextInputStyle,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum TextInputStyle {
    Short,
    Paragraph,
}

impl TextInputStyle {
    pub fn to_serenity(&self) -> serenity::InputTextStyle {
        match self {
            TextInputStyle::Short => serenity::InputTextStyle::Short,
            TextInputStyle::Paragraph => serenity::InputTextStyle::Paragraph,
        }
    }
}

impl TicketMenu {
    pub fn new(id: String, channel_id: String, guild_id: String, category_id: String) -> Self {
        Self {
            id,
            channel_id,
            message_id: None,
            guild_id,
            category_id,
            embed_config: EmbedConfig::default(),
            button_config: ButtonConfig::default(),
            modal_config: ModalConfig::default(),
        }
    }

    pub fn get_key(&self) -> String {
        self.id.clone()
    }

    pub fn to_embed(&self) -> serenity::CreateEmbed {
        let mut embed = serenity::CreateEmbed::new();

        if let Some(ref title) = self.embed_config.title {
            embed = embed.title(title);
        }

        if let Some(ref description) = self.embed_config.description {
            embed = embed.description(description);
        }

        if let Some(ref url) = self.embed_config.url {
            embed = embed.url(url);
        }

        if let Some(color) = self.embed_config.color {
            embed = embed.color(color);
        }

        if let Some(name) = &self.embed_config.author_name {
            let mut author = serenity::CreateEmbedAuthor::new(name);
            if let Some(url) = &self.embed_config.author_url {
                author = author.url(url);
            }
            if let Some(icon) = &self.embed_config.author_icon_url {
                author = author.icon_url(icon);
            }
            embed = embed.author(author);
        }

        if let Some(text) = &self.embed_config.footer_text {
            let mut footer = serenity::CreateEmbedFooter::new(text);
            if let Some(icon) = &self.embed_config.footer_icon_url {
                footer = footer.icon_url(icon);
            }
            embed = embed.footer(footer);
        }

        if let Some(ref image) = self.embed_config.image_url {
            embed = embed.image(image);
        }

        if let Some(ref thumbnail) = self.embed_config.thumbnail_url {
            embed = embed.thumbnail(thumbnail);
        }

        embed.timestamp(serenity::Timestamp::now())
    }

    pub fn to_components(&self) -> Vec<serenity::CreateActionRow> {
        let mut button = serenity::CreateButton::new(format!("ticket_create_{}", self.id))
            .style(serenity::ButtonStyle::Primary)
            .label(&self.button_config.label);

        if let Some(ref emoji_str) = self.button_config.emoji {
            // Try to parse as custom emoji (format: <:name:id> or <a:name:id>)
            if emoji_str.starts_with("<") && emoji_str.ends_with(">") {
                if let Some(emoji) = parse_custom_emoji(emoji_str) {
                    button = button.emoji(emoji);
                }
            } else {
                // Unicode emoji
                button = button.emoji(serenity::ReactionType::Unicode(emoji_str.clone()));
            }
        }

        vec![serenity::CreateActionRow::Buttons(vec![button])]
    }

    /// Post or update the ticket menu message in Discord
    pub async fn post_or_update(&mut self, ctx: &serenity::Context) -> Result<(), Error> {
        let channel_id = serenity::ChannelId::new(self.channel_id.parse::<u64>()?);
        let embed = self.to_embed();
        let components = self.to_components();

        if let Some(ref message_id) = self.message_id {
            // Update existing message
            let message_id_obj = serenity::MessageId::new(message_id.parse::<u64>()?);
            channel_id
                .edit_message(
                    ctx,
                    message_id_obj,
                    serenity::EditMessage::new()
                        .embed(embed)
                        .components(components),
                )
                .await?;
        } else {
            // Create new message
            let message = channel_id
                .send_message(
                    ctx,
                    serenity::CreateMessage::new()
                        .embed(embed)
                        .components(components),
                )
                .await?;
            self.message_id = Some(message.id.get().to_string());
        }

        Ok(())
    }

    /// Post or update the ticket menu message in Discord using HTTP client
    pub async fn post_or_update_with_http(&mut self, http: &serenity::Http) -> Result<(), Error> {
        let channel_id = serenity::ChannelId::new(self.channel_id.parse::<u64>()?);

        // Verify the channel exists and bot has access
        match http.get_channel(channel_id).await {
            Ok(channel) => {
                // Check if it's a text channel
                match channel.guild() {
                    Some(_) => {
                        // Guild channel - proceed
                    }
                    None => {
                        return Err("Channel must be a guild text channel".into());
                    }
                }
            }
            Err(e) => {
                return Err(format!(
                    "Cannot access channel {}: {}. Make sure the bot is in the server and has permission to view this channel.",
                    self.channel_id, e
                ).into());
            }
        }

        let embed = self.to_embed();
        let components = self.to_components();

        if let Some(ref message_id) = self.message_id {
            // Update existing message
            let message_id_obj = serenity::MessageId::new(message_id.parse::<u64>()?);
            let builder = serenity::EditMessage::new()
                .embed(embed)
                .components(components);
            http.edit_message(channel_id, message_id_obj, &builder, vec![])
                .await?;
        } else {
            // Create new message
            let builder = serenity::CreateMessage::new()
                .embed(embed)
                .components(components);
            let message = http.send_message(channel_id, vec![], &builder).await?;
            self.message_id = Some(message.id.get().to_string());
        }

        Ok(())
    }
}

// Helper function to parse custom emoji from Discord format
fn parse_custom_emoji(emoji_str: &str) -> Option<serenity::ReactionType> {
    // Format: <:name:id> or <a:name:id>
    let inner = emoji_str.trim_start_matches('<').trim_end_matches('>');
    let parts: Vec<&str> = inner.split(':').collect();

    if parts.len() >= 3 {
        let animated = parts[0] == "a";
        let name = parts[1];
        let id_str = parts[2];

        if let Ok(id) = id_str.parse::<u64>() {
            return Some(serenity::ReactionType::Custom {
                animated,
                id: serenity::EmojiId::new(id),
                name: Some(name.to_string()),
            });
        }
    }
    None
}

/// Handler for ticket creation button interactions
pub async fn handle_ticket_button(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    // Parse ticket menu ID from custom_id (format: "ticket_create_{id}")
    if let Some(menu_id) = custom_id.strip_prefix("ticket_create_") {
        // Load ticket menu from database
        let db = KV_DATABASE.get().unwrap();
        let tx = db.begin_read()?;
        let table = tx.open_table(TICKETS)?;

        let menu_data = table.get(menu_id)?;
        if menu_data.is_none() {
            drop(table);
            drop(tx);
            interaction
                .create_response(
                    ctx,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content("❌ Ticket menu not found.")
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }

        let ticket_menu: TicketMenu = serde_json::from_str(menu_data.unwrap().value())?;
        drop(table);
        drop(tx);

        // Helper to truncate placeholder to Discord's 100 character limit
        let truncate_placeholder = |s: Option<&String>| -> String {
            s.map(|s| {
                if s.len() > 100 {
                    format!("{}...", &s[..97])
                } else {
                    s.clone()
                }
            })
            .unwrap_or_default()
        };

        // Create and show modal
        let modal = serenity::CreateModal::new(
            format!("ticket_modal_{}", menu_id),
            "Create Support Ticket",
        )
        .components(vec![
            serenity::CreateActionRow::InputText(
                serenity::CreateInputText::new(
                    ticket_menu.modal_config.title_field.style.to_serenity(),
                    &ticket_menu.modal_config.title_field.label,
                    "ticket_title",
                )
                .placeholder(truncate_placeholder(
                    ticket_menu.modal_config.title_field.placeholder.as_ref(),
                ))
                .required(true),
            ),
            serenity::CreateActionRow::InputText(
                serenity::CreateInputText::new(
                    ticket_menu
                        .modal_config
                        .description_field
                        .style
                        .to_serenity(),
                    &ticket_menu.modal_config.description_field.label,
                    "ticket_description",
                )
                .placeholder(truncate_placeholder(
                    ticket_menu
                        .modal_config
                        .description_field
                        .placeholder
                        .as_ref(),
                ))
                .required(true),
            ),
        ]);

        interaction
            .create_response(ctx, serenity::CreateInteractionResponse::Modal(modal))
            .await?;
    }

    Ok(())
}

/// Handler for modal submission
pub async fn handle_ticket_modal(
    ctx: &serenity::Context,
    interaction: &serenity::ModalInteraction,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    // Parse ticket menu ID from custom_id (format: "ticket_modal_{id}")
    if let Some(menu_id) = custom_id.strip_prefix("ticket_modal_") {
        // Load ticket menu from database
        let db = KV_DATABASE.get().unwrap();
        let tx = db.begin_read()?;
        let table = tx.open_table(TICKETS)?;

        let menu_data = table.get(menu_id)?;
        if menu_data.is_none() {
            drop(table);
            drop(tx);
            interaction
                .create_response(
                    ctx,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content("❌ Ticket menu not found.")
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }

        let ticket_menu: TicketMenu = serde_json::from_str(menu_data.unwrap().value())?;
        drop(table);
        drop(tx);

        // Extract modal values
        let title = interaction
            .data
            .components
            .iter()
            .find_map(|row| {
                row.components.iter().find_map(|comp| match comp {
                    serenity::ActionRowComponent::InputText(input) => {
                        if input.custom_id == "ticket_title" {
                            input.value.clone()
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
            })
            .unwrap_or_default();

        let description = interaction
            .data
            .components
            .iter()
            .find_map(|row| {
                row.components.iter().find_map(|comp| match comp {
                    serenity::ActionRowComponent::InputText(input) => {
                        if input.custom_id == "ticket_description" {
                            input.value.clone()
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
            })
            .unwrap_or_default();

        // Defer response
        interaction
            .create_response(
                ctx,
                serenity::CreateInteractionResponse::Defer(
                    serenity::CreateInteractionResponseMessage::new().ephemeral(true),
                ),
            )
            .await?;

        // Create ticket channel
        let guild_id = interaction.guild_id.ok_or("No guild ID")?;
        let user_id = interaction.user.id;

        // Generate ticket number
        let ticket_number = generate_ticket_number()?;
        let channel_name = format!("ticket-{}", ticket_number);

        // Get bot owner ID from env
        let owner_id = *crate::env::AUTHOR_ID;

        // Get bot user ID
        let bot_user_id = ctx.cache.current_user().id;

        // Create channel
        let category_id = serenity::ChannelId::new(ticket_menu.category_id.parse::<u64>()?);

        let channel = guild_id
            .create_channel(
                ctx,
                serenity::CreateChannel::new(&channel_name)
                    .kind(serenity::ChannelType::Text)
                    .category(category_id)
                    .permissions(vec![
                        // Deny everyone
                        serenity::PermissionOverwrite {
                            allow: serenity::Permissions::empty(),
                            deny: serenity::Permissions::VIEW_CHANNEL,
                            kind: serenity::PermissionOverwriteType::Role(guild_id.everyone_role()),
                        },
                        // Allow ticket creator
                        serenity::PermissionOverwrite {
                            allow: serenity::Permissions::VIEW_CHANNEL
                                | serenity::Permissions::SEND_MESSAGES
                                | serenity::Permissions::READ_MESSAGE_HISTORY,
                            deny: serenity::Permissions::empty(),
                            kind: serenity::PermissionOverwriteType::Member(user_id),
                        },
                        // Allow bot owner
                        serenity::PermissionOverwrite {
                            allow: serenity::Permissions::VIEW_CHANNEL
                                | serenity::Permissions::SEND_MESSAGES
                                | serenity::Permissions::READ_MESSAGE_HISTORY
                                | serenity::Permissions::MANAGE_CHANNELS,
                            deny: serenity::Permissions::empty(),
                            kind: serenity::PermissionOverwriteType::Member(serenity::UserId::new(
                                owner_id,
                            )),
                        },
                        // Allow bot
                        serenity::PermissionOverwrite {
                            allow: serenity::Permissions::VIEW_CHANNEL
                                | serenity::Permissions::SEND_MESSAGES
                                | serenity::Permissions::READ_MESSAGE_HISTORY
                                | serenity::Permissions::MANAGE_CHANNELS,
                            deny: serenity::Permissions::empty(),
                            kind: serenity::PermissionOverwriteType::Member(bot_user_id),
                        },
                    ]),
            )
            .await?;

        // Create ticket embed
        let ticket_embed = serenity::CreateEmbed::new()
            .title(format!("🎫 Ticket #{}", ticket_number))
            .description(&description)
            .field("Created by", format!("<@{}>", user_id), true)
            .field("Title", &title, false)
            .color(colors::PRIMARY)
            .timestamp(serenity::Timestamp::now());

        // Post initial message in ticket channel
        channel
            .send_message(
                ctx,
                serenity::CreateMessage::new()
                    .content(format!("<@{}> <@{}>", user_id, owner_id))
                    .embed(ticket_embed),
            )
            .await?;

        // Save active ticket to database
        let active_ticket = ActiveTicket {
            channel_id: channel.id.get().to_string(),
            creator_id: user_id.get().to_string(),
            guild_id: guild_id.get().to_string(),
            ticket_number,
        };
        save_active_ticket(&active_ticket)?;

        // Update deferred response
        interaction
            .edit_response(
                ctx,
                serenity::EditInteractionResponse::new()
                    .content(format!("✅ Ticket created! <#{}>", channel.id)),
            )
            .await?;
    }

    Ok(())
}

/// Generate a unique ticket number
fn generate_ticket_number() -> Result<u32, Error> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    // Use timestamp's last 6 digits
    Ok((now.as_secs() % 1_000_000) as u32)
}

/// Save ticket menu to database
pub fn save_ticket_menu(menu: &TicketMenu) -> Result<(), Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(TICKETS)?;
        let key = menu.get_key();
        let value = serde_json::to_string(menu)?;
        table.insert(key.as_str(), value.as_str())?;
    }
    tx.commit()?;
    Ok(())
}

/// Load ticket menu from database
pub fn load_ticket_menu(id: &str) -> Result<Option<TicketMenu>, Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_read()?;
    let table = tx.open_table(TICKETS)?;

    if let Some(value) = table.get(id)? {
        let menu: TicketMenu = serde_json::from_str(value.value())?;
        Ok(Some(menu))
    } else {
        Ok(None)
    }
}

/// List all ticket menus
pub fn list_ticket_menus() -> Result<Vec<TicketMenu>, Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_read()?;
    let table = tx.open_table(TICKETS)?;

    let mut menus = Vec::new();
    for item in table.range::<&str>(..)? {
        let (_, value) = item?;
        if let Ok(menu) = serde_json::from_str::<TicketMenu>(value.value()) {
            menus.push(menu);
        }
    }

    Ok(menus)
}

/// Delete ticket menu from database
pub fn delete_ticket_menu(id: &str) -> Result<(), Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(TICKETS)?;
        table.remove(id)?;
    }
    tx.commit()?;
    Ok(())
}

/// Active ticket data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveTicket {
    pub channel_id: String,
    pub creator_id: String,
    pub guild_id: String,
    pub ticket_number: u32,
}

/// Store active ticket in database
fn save_active_ticket(ticket: &ActiveTicket) -> Result<(), Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(ACTIVE_TICKETS)?;
        let value = serde_json::to_string(ticket)?;
        table.insert(ticket.channel_id.as_str(), value.as_str())?;
    }
    tx.commit()?;
    Ok(())
}

/// Load active ticket from database by channel ID
fn load_active_ticket(channel_id: &str) -> Result<Option<ActiveTicket>, Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_read()?;
    let table = tx.open_table(ACTIVE_TICKETS)?;

    if let Some(value) = table.get(channel_id)? {
        let ticket: ActiveTicket = serde_json::from_str(value.value())?;
        Ok(Some(ticket))
    } else {
        Ok(None)
    }
}

/// Delete active ticket from database
fn delete_active_ticket(channel_id: &str) -> Result<(), Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(ACTIVE_TICKETS)?;
        let key = channel_id.to_string();
        table.remove(key.as_str())?;
    }
    tx.commit()?;
    Ok(())
}

/// Lock a ticket channel (remove send message permissions for creator)
async fn lock_ticket_channel(
    ctx: &serenity::Context,
    channel_id: serenity::ChannelId,
    _guild_id: serenity::GuildId,
    creator_id: serenity::UserId,
) -> Result<(), Error> {
    // Get current permissions and update to deny sending messages
    let permission_overwrite = serenity::PermissionOverwrite {
        allow: serenity::Permissions::VIEW_CHANNEL | serenity::Permissions::READ_MESSAGE_HISTORY,
        deny: serenity::Permissions::SEND_MESSAGES,
        kind: serenity::PermissionOverwriteType::Member(creator_id),
    };

    channel_id
        .create_permission(ctx, permission_overwrite)
        .await?;

    Ok(())
}

/// Handler for ticket delete button
pub async fn handle_ticket_delete_button(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    // Handle delete confirmation
    if let Some(channel_id_str) = custom_id.strip_prefix("ticket_delete_") {
        if let Ok(channel_id_u64) = channel_id_str.parse::<u64>() {
            let channel_id = serenity::ChannelId::new(channel_id_u64);

            // Check if user is the bot owner
            let user_id = interaction.user.id.get();
            let owner_id = *crate::env::AUTHOR_ID;

            if user_id != owner_id {
                interaction
                    .create_response(
                        ctx,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content("❌ Only the bot owner can delete ticket channels.")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                return Ok(());
            }

            // Delete the channel
            channel_id.delete(ctx).await?;

            // Remove from active tickets database
            let _ = delete_active_ticket(&channel_id_u64.to_string());

            // Respond (even though channel is being deleted)
            interaction
                .create_response(
                    ctx,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content("✅ Ticket channel deleted.")
                            .ephemeral(true),
                    ),
                )
                .await?;
        }
    }
    // Handle close confirmation
    else if let Some(channel_id_str) = custom_id.strip_prefix("ticket_close_confirm_") {
        if let Ok(channel_id_u64) = channel_id_str.parse::<u64>() {
            let channel_id = serenity::ChannelId::new(channel_id_u64);
            let guild_id = interaction.guild_id.ok_or("No guild ID")?;
            let user_id = interaction.user.id;

            // Defer the response
            interaction
                .create_response(
                    ctx,
                    serenity::CreateInteractionResponse::Defer(
                        serenity::CreateInteractionResponseMessage::new().ephemeral(true),
                    ),
                )
                .await?;

            // Close the ticket
            close_ticket(ctx, channel_id, guild_id, user_id).await?;

            // Update response
            interaction
                .edit_response(
                    ctx,
                    serenity::EditInteractionResponse::new()
                        .content("✅ Ticket closed successfully."),
                )
                .await?;
        }
    }
    // Handle close cancellation
    else if custom_id.starts_with("ticket_close_cancel_") {
        interaction
            .create_response(
                ctx,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("❌ Ticket close cancelled.")
                        .ephemeral(true),
                ),
            )
            .await?;
    }

    Ok(())
}

/// Close a ticket channel (lock it and show delete button)
async fn close_ticket(
    ctx: &serenity::Context,
    channel_id: serenity::ChannelId,
    guild_id: serenity::GuildId,
    closed_by: serenity::UserId,
) -> Result<(), Error> {
    // Load active ticket
    let ticket = load_active_ticket(&channel_id.get().to_string())?;
    if ticket.is_none() {
        return Err("Ticket not found in database".into());
    }

    let ticket = ticket.unwrap();
    let creator_id = serenity::UserId::new(ticket.creator_id.parse::<u64>()?);

    // Lock the channel
    lock_ticket_channel(ctx, channel_id, guild_id, creator_id).await?;

    // Send close message with delete button
    let close_embed = serenity::CreateEmbed::new()
        .title("🔒 Ticket Closed")
        .description(format!("This ticket has been closed by <@{}>.", closed_by))
        .color(0xED4245)
        .timestamp(serenity::Timestamp::now());

    let delete_button = serenity::CreateButton::new(format!("ticket_delete_{}", channel_id.get()))
        .style(serenity::ButtonStyle::Danger)
        .label("Delete Channel")
        .emoji('🗑');

    channel_id
        .send_message(
            ctx,
            serenity::CreateMessage::new()
                .embed(close_embed)
                .components(vec![serenity::CreateActionRow::Buttons(vec![
                    delete_button,
                ])]),
        )
        .await?;

    Ok(())
}

/// /close command
#[poise::command(slash_command, guild_only)]
pub async fn close(ctx: crate::Context<'_>) -> Result<(), Error> {
    let channel_id = ctx.channel_id();
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;
    let user_id = ctx.author().id;
    let owner_id = *crate::env::AUTHOR_ID;

    // Check if this is an active ticket channel
    let ticket = load_active_ticket(&channel_id.get().to_string())?;
    if ticket.is_none() {
        ctx.say("❌ This is not an active ticket channel.").await?;
        return Ok(());
    }

    let ticket = ticket.unwrap();

    // If user is the owner, ask for confirmation
    if user_id.get() == owner_id {
        let confirm_embed = serenity::CreateEmbed::new()
            .title("⚠️ Confirm Ticket Close")
            .description("Are you sure you want to close this ticket? The channel will be locked and you'll have the option to delete it.")
            .color(0xFAA61A);

        let confirm_button =
            serenity::CreateButton::new(format!("ticket_close_confirm_{}", channel_id.get()))
                .style(serenity::ButtonStyle::Danger)
                .label("Confirm Close");

        let cancel_button =
            serenity::CreateButton::new(format!("ticket_close_cancel_{}", channel_id.get()))
                .style(serenity::ButtonStyle::Secondary)
                .label("Cancel");

        ctx.send(
            poise::CreateReply::default()
                .embed(confirm_embed)
                .components(vec![serenity::CreateActionRow::Buttons(vec![
                    confirm_button,
                    cancel_button,
                ])]),
        )
        .await?;
    } else if user_id.get() == ticket.creator_id.parse::<u64>()? {
        // If user is the ticket creator, close immediately
        ctx.defer().await?;

        close_ticket(ctx.serenity_context(), channel_id, guild_id, user_id).await?;

        ctx.say("✅ Ticket closed successfully.").await?;
    } else {
        ctx.say("❌ You can only close tickets that you created.")
            .await?;
    }

    Ok(())
}

/// /force-close command (owner only)
#[poise::command(slash_command, guild_only)]
pub async fn force_close(ctx: crate::Context<'_>) -> Result<(), Error> {
    let channel_id = ctx.channel_id();
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;
    let user_id = ctx.author().id;
    let owner_id = *crate::env::AUTHOR_ID;

    // Check if user is the owner
    if user_id.get() != owner_id {
        ctx.say("❌ Only the bot owner can use this command.")
            .await?;
        return Ok(());
    }

    // Check if this is an active ticket channel
    let ticket = load_active_ticket(&channel_id.get().to_string())?;
    if ticket.is_none() {
        ctx.say("❌ This is not an active ticket channel.").await?;
        return Ok(());
    }

    ctx.defer().await?;

    close_ticket(ctx.serenity_context(), channel_id, guild_id, user_id).await?;

    ctx.say("✅ Ticket force-closed successfully.").await?;

    Ok(())
}
