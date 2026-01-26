use crate::colors;
use crate::{Context, Error, KV_DATABASE, PAPERS};
use poise::serenity_prelude as serenity;
use redb::ReadableDatabase;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PapersState {
    pub channel_id: u64,
    pub message_id: u64,
    pub guild_id: Option<u64>,
    pub embed_config: EmbedConfig,
    pub buttons: Vec<ButtonConfig>,
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
            title: Some("📋 Get Your Roles".to_string()),
            description: Some("Click the buttons below to get your roles!".to_string()),
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
pub enum ButtonType {
    Role { role_id: u64 },
    Link { url: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ButtonConfig {
    pub label: String,
    pub button_type: ButtonType,
    pub style: ButtonStyle,
    pub emoji: Option<String>, // Partial emoji: either unicode emoji or custom emoji format
}

// Old ButtonConfig structure for migration (v0)
#[derive(Debug, Deserialize)]
struct ButtonConfigV0 {
    pub name: String, // Old field name
    pub role_id: u64, // Old structure had role_id directly
    #[serde(default)]
    pub style: ButtonStyle,
    #[serde(default)]
    pub emoji: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default)]
pub enum ButtonStyle {
    #[default]
    Primary,
    Secondary,
    Success,
    Danger,
}

impl ButtonStyle {
    pub fn to_serenity(&self) -> serenity::ButtonStyle {
        match self {
            ButtonStyle::Primary => serenity::ButtonStyle::Primary,
            ButtonStyle::Secondary => serenity::ButtonStyle::Secondary,
            ButtonStyle::Success => serenity::ButtonStyle::Success,
            ButtonStyle::Danger => serenity::ButtonStyle::Danger,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "primary" => Some(ButtonStyle::Primary),
            "secondary" => Some(ButtonStyle::Secondary),
            "success" => Some(ButtonStyle::Success),
            "danger" => Some(ButtonStyle::Danger),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ButtonStyle::Primary => "Primary",
            ButtonStyle::Secondary => "Secondary",
            ButtonStyle::Success => "Success",
            ButtonStyle::Danger => "Danger",
        }
    }
}

impl PapersState {
    pub fn new(channel_id: u64, message_id: u64, guild_id: Option<u64>) -> Self {
        Self {
            channel_id,
            message_id,
            guild_id,
            embed_config: EmbedConfig::default(),
            buttons: Vec::new(),
        }
    }

    pub fn get_key(&self) -> String {
        format!("{}_{}", self.channel_id, self.message_id)
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
        if self.buttons.is_empty() {
            return Vec::new();
        }

        let buttons: Vec<serenity::CreateButton> = self
            .buttons
            .iter()
            .map(|btn| {
                let mut button = match &btn.button_type {
                    ButtonType::Role { role_id } => {
                        serenity::CreateButton::new(format!("papers_role_{}", role_id))
                            .style(btn.style.to_serenity())
                    }
                    ButtonType::Link { url } => serenity::CreateButton::new_link(url),
                };

                button = button.label(&btn.label);

                if let Some(ref emoji_str) = btn.emoji {
                    // Try to parse as custom emoji (format: <:name:id> or <a:name:id>)
                    if emoji_str.starts_with("<") && emoji_str.ends_with(">") {
                        // Custom emoji format
                        if let Some(emoji) = parse_custom_emoji(emoji_str) {
                            button = button.emoji(emoji);
                        }
                    } else {
                        // Unicode emoji
                        button = button.emoji(serenity::ReactionType::Unicode(emoji_str.clone()));
                    }
                }

                button
            })
            .collect();

        // Split into rows of max 5 buttons each
        buttons
            .chunks(5)
            .map(|chunk| serenity::CreateActionRow::Buttons(chunk.to_vec()))
            .collect()
    }

    pub fn find_button_by_identifier(&self, identifier: &str) -> Option<usize> {
        // Try to parse as index first
        if let Ok(index) = identifier.parse::<usize>()
            && index < self.buttons.len()
        {
            return Some(index);
        }
        // Otherwise search by label
        self.buttons.iter().position(|b| b.label == identifier)
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

pub fn load_papers_state_by_channel(channel_id: u64) -> Result<Option<PapersState>, Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_read()?;
    let table = tx.open_table(PAPERS)?;

    // Scan for any state in this channel
    for item in table.range::<&str>(..)? {
        let (_, value) = item?;

        // Try to deserialize with migration support
        let state: PapersState = match serde_json::from_str(value.value()) {
            Ok(s) => s,
            Err(_) => {
                // Try to migrate from old format
                match try_migrate_papers_state(value.value()) {
                    Ok(migrated) => {
                        // Save migrated version
                        drop(table);
                        drop(tx);
                        save_papers_state(&migrated)?;
                        return Ok(Some(migrated));
                    }
                    Err(e) => {
                        return Err(format!("Failed to load or migrate papers state: {}", e).into());
                    }
                }
            }
        };

        if state.channel_id == channel_id {
            return Ok(Some(state));
        }
    }

    Ok(None)
}

// Migration function from old JSON format
fn try_migrate_papers_state(json: &str) -> Result<PapersState, Error> {
    // Try parsing as old format with ButtonConfigV0
    #[derive(Deserialize)]
    struct PapersStateV0 {
        channel_id: u64,
        message_id: u64,
        guild_id: Option<u64>,
        embed_config: EmbedConfig,
        buttons: Vec<ButtonConfigV0>,
    }

    let old_state: PapersStateV0 = serde_json::from_str(json)?;

    Ok(PapersState {
        channel_id: old_state.channel_id,
        message_id: old_state.message_id,
        guild_id: old_state.guild_id,
        embed_config: old_state.embed_config,
        buttons: old_state
            .buttons
            .into_iter()
            .map(|b| ButtonConfig {
                label: b.name,
                button_type: ButtonType::Role { role_id: b.role_id },
                style: b.style,
                emoji: b.emoji,
            })
            .collect(),
    })
}

pub fn save_papers_state(state: &PapersState) -> Result<(), Error> {
    let key = state.get_key();
    let value = serde_json::to_string(state)?;
    crate::db::write_entry(PAPERS, &key, &value)
        .map_err(|e| format!("Failed to save papers state: {}", e).into())
}

pub fn delete_papers_state(channel_id: u64, message_id: u64) -> Result<(), Error> {
    let key = format!("{}_{}", channel_id, message_id);
    crate::db::delete_entry(PAPERS, &key)
        .map_err(|e| format!("Failed to delete papers state: {}", e).into())
}

async fn update_papers_message(http: &serenity::Http, state: &PapersState) -> Result<(), Error> {
    let channel_id = serenity::ChannelId::new(state.channel_id);
    let message_id = serenity::MessageId::new(state.message_id);

    let embed = state.to_embed();
    let components = state.to_components();

    let builder = serenity::EditMessage::new()
        .embed(embed)
        .components(components);

    channel_id.edit_message(http, message_id, builder).await?;

    Ok(())
}

// Helper to delete caller message for prefix commands
async fn delete_caller_message(ctx: &Context<'_>) -> Result<(), Error> {
    if let poise::Context::Prefix(pctx) = ctx {
        let _ = pctx.msg.delete(&pctx.serenity_context.http).await;
    }
    Ok(())
}

// Helper to create a simple embed with title and description
fn create_embed(title: &str, description: &str, color: u32) -> serenity::CreateEmbed {
    serenity::CreateEmbed::new()
        .title(title)
        .description(description)
        .color(color)
        .timestamp(serenity::Timestamp::now())
}

// Helper to send an embed and clean up caller message
async fn send_embed_reply(
    ctx: &Context<'_>,
    title: &str,
    description: &str,
    color: u32,
    ephemeral: bool,
) -> Result<(), Error> {
    let embed = create_embed(title, description, color);
    let mut reply = poise::CreateReply::default().embed(embed);
    if ephemeral {
        reply = reply.ephemeral(true);
    }
    ctx.send(reply).await?;
    delete_caller_message(ctx).await?;
    Ok(())
}

// Helper to load papers state with automatic error handling
async fn require_papers_state(
    ctx: &Context<'_>,
    channel_id: u64,
    use_start_hint: bool,
) -> Result<Option<PapersState>, Error> {
    match load_papers_state_by_channel(channel_id)? {
        Some(state) => Ok(Some(state)),
        None => {
            let description = if use_start_hint {
                "No papers message found in this channel. Use `/papers start` first."
            } else {
                "No papers message found in this channel."
            };
            send_embed_reply(
                ctx,
                "❌ No Papers Message",
                description,
                colors::ERROR,
                false,
            )
            .await?;
            Ok(None)
        }
    }
}

// Helper to check if modal should be used (slash command only)
async fn require_slash_command(ctx: &Context<'_>) -> Result<bool, Error> {
    if matches!(ctx, poise::Context::Prefix(_)) {
        send_embed_reply(
            ctx,
            "❌ Slash Command Only",
            "This command only works as a slash command to use modals.",
            colors::ERROR,
            false,
        )
        .await?;
        return Ok(false);
    }
    Ok(true)
}

/// Manage role assignment papers with buttons
#[poise::command(
    prefix_command,
    slash_command,
    subcommands(
        "start",
        "button",
        "link",
        "editbutton",
        "delbutton",
        "author",
        "body",
        "images",
        "stop",
        "prune"
    ),
    subcommand_required,
    check = "crate::permissions::check_admin",
    category = "Utility"
)]
pub async fn papers(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Create a new papers message in this channel
#[poise::command(prefix_command, slash_command)]
async fn start(ctx: Context<'_>) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();
    let guild_id = ctx.guild_id().map(|g| g.get());

    // Check if papers already exists in this channel
    if let Some(existing) = load_papers_state_by_channel(channel_id)? {
        send_embed_reply(
            &ctx,
            "❌ Already Exists",
            &format!(
                "Papers already exists in this channel. Message ID: {}",
                existing.message_id
            ),
            colors::ERROR,
            true,
        )
        .await?;
        return Ok(());
    }

    // Create the initial message
    let state = PapersState::new(channel_id, 0, guild_id);
    let embed = state.to_embed();

    let sent_message = ctx
        .channel_id()
        .send_message(
            ctx.serenity_context(),
            serenity::CreateMessage::new().embed(embed),
        )
        .await?;

    // Update state with message ID and save
    let mut state = state;
    state.message_id = sent_message.id.get();
    save_papers_state(&state)?;

    send_embed_reply(
        &ctx,
        "✅ Papers Created",
        &format!("Created papers message with ID: {}", sent_message.id),
        colors::SUCCESS,
        true,
    )
    .await?;

    Ok(())
}

/// Add a button to assign a role
#[poise::command(prefix_command, slash_command)]
async fn button(
    ctx: Context<'_>,
    #[description = "Button label"] name: String,
    #[description = "Role to assign"] role: serenity::Role,
) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();

    // Load existing state
    let Some(mut state) = require_papers_state(&ctx, channel_id, true).await? else {
        return Ok(());
    };

    // Check if button with this label already exists
    if state.buttons.iter().any(|b| b.label == name) {
        send_embed_reply(
            &ctx,
            "❌ Button Exists",
            &format!("A button labeled '{}' already exists.", name),
            colors::ERROR,
            false,
        )
        .await?;
        return Ok(());
    }

    // Add button
    state.buttons.push(ButtonConfig {
        label: name.clone(),
        button_type: ButtonType::Role {
            role_id: role.id.get(),
        },
        style: ButtonStyle::Primary,
        emoji: None,
    });

    // Save and update message
    save_papers_state(&state)?;
    update_papers_message(ctx.serenity_context().http.as_ref(), &state).await?;

    send_embed_reply(
        &ctx,
        "✅ Button Added",
        &format!("Added button '{}' for role <@&{}>", name, role.id),
        colors::SUCCESS,
        true,
    )
    .await?;

    Ok(())
}

/// Add a link button
#[poise::command(prefix_command, slash_command)]
async fn link(
    ctx: Context<'_>,
    #[description = "Button label"] text: String,
    #[description = "URL to link to"] url: String,
) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();

    // Load existing state
    let Some(mut state) = require_papers_state(&ctx, channel_id, true).await? else {
        return Ok(());
    };

    // Check if button with this label already exists
    if state.buttons.iter().any(|b| b.label == text) {
        send_embed_reply(
            &ctx,
            "❌ Button Exists",
            &format!("A button labeled '{}' already exists.", text),
            colors::ERROR,
            false,
        )
        .await?;
        return Ok(());
    }

    // Validate URL
    if !url.starts_with("http://") && !url.starts_with("https://") {
        send_embed_reply(
            &ctx,
            "❌ Invalid URL",
            "URL must start with http:// or https://",
            colors::ERROR,
            false,
        )
        .await?;
        return Ok(());
    }

    // Add link button (link buttons always use Link style, no custom style)
    state.buttons.push(ButtonConfig {
        label: text.clone(),
        button_type: ButtonType::Link { url: url.clone() },
        style: ButtonStyle::Primary, // Not used for link buttons
        emoji: None,
    });

    // Save and update message
    save_papers_state(&state)?;
    update_papers_message(ctx.serenity_context().http.as_ref(), &state).await?;

    send_embed_reply(
        &ctx,
        "✅ Link Button Added",
        &format!("Added link button '{}' → {}", text, url),
        colors::SUCCESS,
        false,
    )
    .await?;

    Ok(())
}

/// Edit an existing button
#[poise::command(prefix_command, slash_command)]
async fn editbutton(
    ctx: Context<'_>,
    #[description = "Button name or index (0-based) to edit"] identifier: String,
) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();

    // Load existing state
    let Some(state) = require_papers_state(&ctx, channel_id, false).await? else {
        return Ok(());
    };

    // Find button
    let button_index = match state.find_button_by_identifier(&identifier) {
        Some(idx) => idx,
        None => {
            let mut description = format!("No button found with identifier '{}'.\n\n", identifier);

            if state.buttons.is_empty() {
                description.push_str("There are no buttons on this message.");
            } else {
                description.push_str("**Available buttons:**\n");
                for (i, btn) in state.buttons.iter().enumerate() {
                    let btn_type = match &btn.button_type {
                        ButtonType::Role { .. } => "Role",
                        ButtonType::Link { .. } => "Link",
                    };
                    description.push_str(&format!("[{}] {} ({})\n", i, btn.label, btn_type));
                }
            }

            send_embed_reply(
                &ctx,
                "❌ Button Not Found",
                &description,
                colors::ERROR,
                false,
            )
            .await?;
            return Ok(());
        }
    };

    let button = &state.buttons[button_index];
    let is_link_button = matches!(button.button_type, ButtonType::Link { .. });

    if !require_slash_command(&ctx).await? {
        return Ok(());
    }

    // Prefill modal values
    let current_label = button.label.clone();
    let current_style = button.style.as_str();
    let current_emoji = button.emoji.clone().unwrap_or_default();

    match ctx {
        poise::Context::Application(app_ctx) => {
            let mut modal = serenity::CreateQuickModal::new("Edit Button")
                .timeout(std::time::Duration::from_secs(600))
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Label",
                        "label",
                    )
                    .value(&current_label),
                );

            if !is_link_button {
                modal = modal.field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Style (Primary/Secondary/Success/Danger)",
                        "style",
                    )
                    .value(current_style),
                );
            }

            modal = modal.field(
                serenity::CreateInputText::new(
                    serenity::InputTextStyle::Short,
                    "Emoji (unicode or <:name:id>)",
                    "emoji",
                )
                .value(&current_emoji)
                .required(false),
            );

            if let Some(response) = app_ctx
                .interaction
                .quick_modal(app_ctx.serenity_context, modal)
                .await?
            {
                let inputs = &response.inputs;

                let new_label = inputs
                    .first()
                    .and_then(|s| if s.is_empty() { None } else { Some(s.clone()) });
                let new_style = if is_link_button {
                    button.style // Keep existing style for link buttons
                } else {
                    inputs
                        .get(1)
                        .and_then(|s| ButtonStyle::from_str(s))
                        .unwrap_or(button.style)
                };
                let new_emoji = inputs
                    .get(if is_link_button { 1 } else { 2 })
                    .and_then(|s| if s.is_empty() { None } else { Some(s.clone()) });

                // Update state
                let mut state = load_papers_state_by_channel(channel_id)?.unwrap();
                if let Some(btn) = state.buttons.get_mut(button_index) {
                    if let Some(label) = new_label {
                        btn.label = label;
                    }
                    btn.style = new_style;
                    btn.emoji = new_emoji;
                }

                save_papers_state(&state)?;
                update_papers_message(app_ctx.serenity_context.http.as_ref(), &state).await?;

                response
                    .interaction
                    .create_response(
                        app_ctx.serenity_context,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .embed(
                                    serenity::CreateEmbed::new()
                                        .title("✅ Button Updated")
                                        .description(format!(
                                            "Updated button at index {}",
                                            button_index
                                        ))
                                        .color(colors::SUCCESS)
                                        .timestamp(serenity::Timestamp::now()),
                                )
                                .ephemeral(true),
                        ),
                    )
                    .await?;
            }
        }
        poise::Context::Prefix(_) => unreachable!(),
    }

    delete_caller_message(&ctx).await?;
    Ok(())
}

/// Remove a button by name or 0-indexed position
#[poise::command(prefix_command, slash_command)]
async fn delbutton(
    ctx: Context<'_>,
    #[description = "Button name or index (0-based) to remove"] identifier: String,
) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();

    // Load existing state
    let Some(mut state) = require_papers_state(&ctx, channel_id, false).await? else {
        return Ok(());
    };

    // Try to parse as index first, otherwise treat as name
    let removal_result = if let Ok(index) = identifier.parse::<usize>() {
        // Remove by index
        if index < state.buttons.len() {
            let removed_button = state.buttons.remove(index);
            Some((index, removed_button.label))
        } else {
            None
        }
    } else {
        // Remove by label
        if let Some(pos) = state.buttons.iter().position(|b| b.label == identifier) {
            let removed_button = state.buttons.remove(pos);
            Some((pos, removed_button.label))
        } else {
            None
        }
    };

    match removal_result {
        Some((index, button_name)) => {
            // Save and update message
            save_papers_state(&state)?;
            update_papers_message(ctx.serenity_context().http.as_ref(), &state).await?;

            send_embed_reply(
                &ctx,
                "✅ Button Removed",
                &format!("Removed button '{}' at index {}", button_name, index),
                colors::SUCCESS,
                true,
            )
            .await?;
        }
        None => {
            // Build helpful error message showing available buttons
            let mut description = if let Ok(index) = identifier.parse::<usize>() {
                format!("No button at index {}.\n\n", index)
            } else {
                format!("No button named '{}'.\n\n", identifier)
            };

            if state.buttons.is_empty() {
                description.push_str("There are no buttons on this message.");
            } else {
                description.push_str("**Available buttons:**\n");
                for (i, btn) in state.buttons.iter().enumerate() {
                    let btn_type = match &btn.button_type {
                        ButtonType::Role { .. } => "Role",
                        ButtonType::Link { .. } => "Link",
                    };
                    description.push_str(&format!(
                        "[{}] {} ({})
",
                        i, btn.label, btn_type
                    ));
                }
            }

            send_embed_reply(
                &ctx,
                "❌ Button Not Found",
                &description,
                colors::ERROR,
                false,
            )
            .await?;
        }
    }

    delete_caller_message(&ctx).await?;
    Ok(())
}

/// Update the author field using a modal
#[poise::command(prefix_command, slash_command)]
async fn author(ctx: Context<'_>) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();

    // Load existing state to prefill modal
    let Some(state) = require_papers_state(&ctx, channel_id, false).await? else {
        return Ok(());
    };

    if !require_slash_command(&ctx).await? {
        return Ok(());
    }

    // Create modal with prefilled values
    let author_name = state.embed_config.author_name.clone().unwrap_or_default();
    let author_url = state.embed_config.author_url.clone().unwrap_or_default();

    match ctx {
        poise::Context::Application(app_ctx) => {
            let modal = serenity::CreateQuickModal::new("Edit Author")
                .timeout(std::time::Duration::from_secs(600))
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Author Name",
                        "author_name",
                    )
                    .value(&author_name),
                )
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Author URL (optional)",
                        "author_url",
                    )
                    .value(&author_url)
                    .required(false),
                );

            if let Some(response) = app_ctx
                .interaction
                .quick_modal(app_ctx.serenity_context, modal)
                .await?
            {
                let inputs = &response.inputs;
                let new_author_name = inputs
                    .first()
                    .and_then(|s| if s.is_empty() { None } else { Some(s.clone()) });
                let new_author_url = inputs
                    .get(1)
                    .and_then(|s| if s.is_empty() { None } else { Some(s.clone()) });

                // Update state
                let mut state = load_papers_state_by_channel(channel_id)?.unwrap();
                state.embed_config.author_name = new_author_name.clone();
                state.embed_config.author_url = new_author_url.clone();

                save_papers_state(&state)?;
                update_papers_message(app_ctx.serenity_context.http.as_ref(), &state).await?;

                let description = match new_author_name {
                    Some(name) => format!("Author updated to: {}", name),
                    None => "Author cleared".to_string(),
                };

                response
                    .interaction
                    .create_response(
                        app_ctx.serenity_context,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .embed(
                                    serenity::CreateEmbed::new()
                                        .title("✅ Author Updated")
                                        .description(description)
                                        .color(colors::SUCCESS)
                                        .timestamp(serenity::Timestamp::now()),
                                )
                                .ephemeral(true),
                        ),
                    )
                    .await?;
            }
        }
        poise::Context::Prefix(_) => unreachable!(),
    }

    delete_caller_message(&ctx).await?;
    Ok(())
}

/// Update body fields using a modal
#[poise::command(prefix_command, slash_command)]
async fn body(ctx: Context<'_>) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();

    // Load existing state to prefill modal
    let Some(state) = require_papers_state(&ctx, channel_id, false).await? else {
        return Ok(());
    };

    if !require_slash_command(&ctx).await? {
        return Ok(());
    }

    // Prefill values
    let title = state.embed_config.title.clone().unwrap_or_default();
    let description = state.embed_config.description.clone().unwrap_or_default();
    let url = state.embed_config.url.clone().unwrap_or_default();
    let color_hex = state
        .embed_config
        .color
        .map(|c| format!("{:06X}", c))
        .unwrap_or_default();
    let footer = state.embed_config.footer_text.clone().unwrap_or_default();

    match ctx {
        poise::Context::Application(app_ctx) => {
            let modal = serenity::CreateQuickModal::new("Edit Body")
                .timeout(std::time::Duration::from_secs(600))
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Title",
                        "title",
                    )
                    .value(&title)
                    .required(false),
                )
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Paragraph,
                        "Description",
                        "description",
                    )
                    .value(&description)
                    .required(false),
                )
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "URL (optional)",
                        "url",
                    )
                    .value(&url)
                    .required(false),
                )
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Color (hex, e.g., FF5733)",
                        "color",
                    )
                    .value(&color_hex)
                    .required(false),
                )
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Footer Text",
                        "footer",
                    )
                    .value(&footer)
                    .required(false),
                );

            if let Some(response) = app_ctx
                .interaction
                .quick_modal(app_ctx.serenity_context, modal)
                .await?
            {
                let inputs = &response.inputs;

                // Parse inputs
                let new_title = inputs
                    .first()
                    .and_then(|s| if s.is_empty() { None } else { Some(s.clone()) });
                let new_description = inputs
                    .get(1)
                    .and_then(|s| if s.is_empty() { None } else { Some(s.clone()) });
                let new_url = inputs
                    .get(2)
                    .and_then(|s| if s.is_empty() { None } else { Some(s.clone()) });
                let new_color = inputs.get(3).and_then(|s| {
                    if s.is_empty() {
                        return None;
                    }
                    u32::from_str_radix(s.trim_start_matches('#'), 16).ok()
                });
                let new_footer = inputs
                    .get(4)
                    .and_then(|s| if s.is_empty() { None } else { Some(s.clone()) });

                // Update state
                let mut state = load_papers_state_by_channel(channel_id)?.unwrap();
                state.embed_config.title = new_title;
                state.embed_config.description = new_description;
                state.embed_config.url = new_url;
                state.embed_config.color = new_color;
                state.embed_config.footer_text = new_footer;

                save_papers_state(&state)?;
                update_papers_message(app_ctx.serenity_context.http.as_ref(), &state).await?;

                response
                    .interaction
                    .create_response(
                        app_ctx.serenity_context,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .embed(
                                    serenity::CreateEmbed::new()
                                        .title("✅ Body Updated")
                                        .description("Embed body fields have been updated.")
                                        .color(colors::SUCCESS)
                                        .timestamp(serenity::Timestamp::now()),
                                )
                                .ephemeral(true),
                        ),
                    )
                    .await?;
            }
        }
        poise::Context::Prefix(_) => unreachable!(),
    }

    delete_caller_message(&ctx).await?;
    Ok(())
}

/// Update image fields using a modal
#[poise::command(prefix_command, slash_command)]
async fn images(ctx: Context<'_>) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();

    // Load existing state to prefill modal
    let Some(state) = require_papers_state(&ctx, channel_id, false).await? else {
        return Ok(());
    };

    if !require_slash_command(&ctx).await? {
        return Ok(());
    }

    // Prefill values
    let image_url = state.embed_config.image_url.clone().unwrap_or_default();
    let thumbnail_url = state.embed_config.thumbnail_url.clone().unwrap_or_default();
    let author_icon = state
        .embed_config
        .author_icon_url
        .clone()
        .unwrap_or_default();
    let footer_icon = state
        .embed_config
        .footer_icon_url
        .clone()
        .unwrap_or_default();

    match ctx {
        poise::Context::Application(app_ctx) => {
            let modal = serenity::CreateQuickModal::new("Edit Images")
                .timeout(std::time::Duration::from_secs(600))
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Image URL (main image)",
                        "image_url",
                    )
                    .value(&image_url)
                    .required(false),
                )
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Thumbnail URL",
                        "thumbnail_url",
                    )
                    .value(&thumbnail_url)
                    .required(false),
                )
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Author Icon URL",
                        "author_icon_url",
                    )
                    .value(&author_icon)
                    .required(false),
                )
                .field(
                    serenity::CreateInputText::new(
                        serenity::InputTextStyle::Short,
                        "Footer Icon URL",
                        "footer_icon_url",
                    )
                    .value(&footer_icon)
                    .required(false),
                );

            if let Some(response) = app_ctx
                .interaction
                .quick_modal(app_ctx.serenity_context, modal)
                .await?
            {
                let inputs = &response.inputs;

                // Parse inputs
                let new_image_url = inputs
                    .first()
                    .and_then(|s| if s.is_empty() { None } else { Some(s.clone()) });
                let new_thumbnail_url = inputs
                    .get(1)
                    .and_then(|s| if s.is_empty() { None } else { Some(s.clone()) });
                let new_author_icon = inputs
                    .get(2)
                    .and_then(|s| if s.is_empty() { None } else { Some(s.clone()) });
                let new_footer_icon = inputs
                    .get(3)
                    .and_then(|s| if s.is_empty() { None } else { Some(s.clone()) });

                // Update state
                let mut state = load_papers_state_by_channel(channel_id)?.unwrap();
                state.embed_config.image_url = new_image_url;
                state.embed_config.thumbnail_url = new_thumbnail_url;
                state.embed_config.author_icon_url = new_author_icon;
                state.embed_config.footer_icon_url = new_footer_icon;

                save_papers_state(&state)?;
                update_papers_message(app_ctx.serenity_context.http.as_ref(), &state).await?;

                response
                    .interaction
                    .create_response(
                        app_ctx.serenity_context,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .embed(
                                    serenity::CreateEmbed::new()
                                        .title("✅ Images Updated")
                                        .description("Embed image fields have been updated.")
                                        .color(colors::SUCCESS)
                                        .timestamp(serenity::Timestamp::now()),
                                )
                                .ephemeral(true),
                        ),
                    )
                    .await?;
            }
        }
        poise::Context::Prefix(_) => unreachable!(),
    }

    delete_caller_message(&ctx).await?;
    Ok(())
}

/// Delete the papers message and remove from database
#[poise::command(prefix_command, slash_command)]
async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();

    // Load existing state
    let Some(state) = require_papers_state(&ctx, channel_id, false).await? else {
        return Ok(());
    };

    // Delete the message
    let channel_id_obj = serenity::ChannelId::new(state.channel_id);
    let message_id = serenity::MessageId::new(state.message_id);

    let _ = channel_id_obj
        .delete_message(ctx.serenity_context().http.as_ref(), message_id)
        .await;

    // Remove from database
    delete_papers_state(state.channel_id, state.message_id)?;

    send_embed_reply(
        &ctx,
        "✅ Papers Removed",
        "Deleted papers message and removed from database.",
        colors::SUCCESS,
        true,
    )
    .await?;

    Ok(())
}

/// Clean up papers messages that no longer exist
#[poise::command(prefix_command, slash_command)]
async fn prune(ctx: Context<'_>) -> Result<(), Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_read()?;
    let table = tx.open_table(PAPERS)?;

    let mut to_delete = Vec::new();
    let mut checked = 0;
    let mut deleted = 0;
    let mut corrupted = 0;

    for item in table.range::<&str>(..)? {
        let (key, value) = item?;

        // Try to parse the state, if it fails mark as corrupted
        let state = match serde_json::from_str::<PapersState>(value.value()) {
            Ok(s) => s,
            Err(_) => {
                // Try migration
                match try_migrate_papers_state(value.value()) {
                    Ok(s) => s,
                    Err(_) => {
                        // Corrupted data, mark for deletion
                        corrupted += 1;
                        to_delete.push(key.value().to_string());
                        continue;
                    }
                }
            }
        };

        checked += 1;

        // Try to fetch the message
        let channel_id = serenity::ChannelId::new(state.channel_id);
        let message_id = serenity::MessageId::new(state.message_id);

        if channel_id
            .message(ctx.serenity_context().http.as_ref(), message_id)
            .await
            .is_err()
        {
            to_delete.push(key.value().to_string());
        }
    }

    drop(table);
    drop(tx);

    // Delete inaccessible entries
    if !to_delete.is_empty() {
        let tx = db.begin_write()?;
        {
            let mut table = tx.open_table(PAPERS)?;
            for key in &to_delete {
                let _ = table.remove(key.as_str());
                deleted += 1;
            }
        }
        tx.commit()?;
    }

    let mut description = format!(
        "Checked {} valid papers messages.\nDeleted {} inaccessible entries.",
        checked, deleted
    );

    if corrupted > 0 {
        description.push_str(&format!(
            "\nCleaned up {} corrupted/mismatched entries.",
            corrupted
        ));
    }

    send_embed_reply(
        &ctx,
        "✅ Prune Complete",
        &description,
        colors::SUCCESS,
        false,
    )
    .await?;

    Ok(())
}

// Handler for button interactions
pub async fn handle_papers_button(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    // Parse role ID from custom_id (format: "papers_role_{role_id}")
    if let Some(role_id_str) = custom_id.strip_prefix("papers_role_")
        && let Ok(role_id) = role_id_str.parse::<u64>()
    {
        let guild_id = match interaction.guild_id {
            Some(g) => g,
            None => {
                interaction
                    .create_response(
                        ctx,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content("❌ This can only be used in a server.")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                return Ok(());
            }
        };

        let user_id = interaction.user.id;
        let role_id_obj = serenity::RoleId::new(role_id);

        // Try to add the role
        match guild_id.member(ctx, user_id).await {
            Ok(_member) => {
                match ctx
                    .http
                    .add_member_role(guild_id, user_id, role_id_obj, None)
                    .await
                {
                    Ok(_) => {
                        interaction
                            .create_response(
                                ctx,
                                serenity::CreateInteractionResponse::Message(
                                    serenity::CreateInteractionResponseMessage::new()
                                        .content(format!("✅ Role <@&{}> assigned!", role_id))
                                        .ephemeral(true),
                                ),
                            )
                            .await?;
                    }
                    Err(e) => {
                        interaction
                            .create_response(
                                ctx,
                                serenity::CreateInteractionResponse::Message(
                                    serenity::CreateInteractionResponseMessage::new()
                                        .content(format!("❌ Failed to assign role: {}", e))
                                        .ephemeral(true),
                                ),
                            )
                            .await?;
                    }
                }
            }
            Err(e) => {
                interaction
                    .create_response(
                        ctx,
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content(format!("❌ Failed to get member: {}", e))
                                .ephemeral(true),
                        ),
                    )
                    .await?;
            }
        }
    }

    Ok(())
}
