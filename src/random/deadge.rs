use crate::colors;
use crate::{AYDY, Context, Error, KV_DATABASE};
use poise::serenity_prelude as serenity;
use redb::{ReadableDatabase, ReadableTable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{Duration, interval};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AydyState {
    pub channel_id: u64,
    pub guild_id: Option<u64>,
    pub message_id: Option<u64>,
    pub last_sent: i64,                           // Unix timestamp
    pub enrolled_users: HashMap<u64, UserStatus>, // user_id -> UserStatus
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserStatus {
    pub user_name: String,
    pub last_response: i64, // Unix timestamp
    pub enrolled_at: i64,   // Unix timestamp
}

impl AydyState {
    pub fn new(channel_id: u64, guild_id: Option<u64>) -> Self {
        Self {
            channel_id,
            guild_id,
            message_id: None,
            last_sent: chrono::Utc::now().timestamp(),
            enrolled_users: HashMap::new(),
        }
    }

    pub fn get_non_responders(&self, hours: i64) -> Vec<(u64, &UserStatus)> {
        let cutoff = chrono::Utc::now().timestamp() - (hours * 3600);
        self.enrolled_users
            .iter()
            .filter(|(_, status)| status.last_response < cutoff)
            .map(|(id, status)| (*id, status))
            .collect()
    }

    pub fn update_user_response(&mut self, user_id: u64, user_name: String) {
        let now = chrono::Utc::now().timestamp();
        if let Some(status) = self.enrolled_users.get_mut(&user_id) {
            status.last_response = now;
            status.user_name = user_name; // Update name in case it changed
        } else {
            // First time user is responding
            self.enrolled_users.insert(
                user_id,
                UserStatus {
                    user_name,
                    last_response: now,
                    enrolled_at: now,
                },
            );
        }
    }
}

/// Start the "Are you dead yet?" check
#[poise::command(
    prefix_command,
    slash_command,
    subcommands("start", "stop", "status"),
    category = "AYDY"
)]
pub async fn aydy(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Start the AYDY check in this channel
#[poise::command(prefix_command, slash_command)]
async fn start(ctx: Context<'_>) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();
    let guild_id = ctx.guild_id().map(|g| g.get());

    // Check if AYDY is already running in this channel
    let db = KV_DATABASE.get().unwrap();
    let key = format!("aydy_{}", channel_id);

    {
        let tx = db.begin_read()?;
        let table = tx.open_table(AYDY)?;
        if table.get(key.as_str())?.is_some() {
            let embed = serenity::CreateEmbed::new()
                .title("❌ Already Running")
                .description(
                    "AYDY is already running in this channel. Use `/aydy stop` to stop it first.",
                )
                .color(colors::ERROR)
                .timestamp(serenity::Timestamp::now());
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    }

    // Create new AYDY state
    let state = AydyState::new(channel_id, guild_id);

    // Send initial message
    let message = send_aydy_message(&ctx.serenity_context().http, &state).await?;

    // Update state with message ID
    let mut state = state;
    state.message_id = Some(message.id.get());

    // Save to database
    {
        let tx = db.begin_write()?;
        {
            let mut table = tx.open_table(AYDY)?;
            let value = serde_json::to_string(&state)?;
            table.insert(key.as_str(), value.as_str())?;
        }
        tx.commit()?;
    }

    let embed = serenity::CreateEmbed::new()
        .title("✅ AYDY Check Started")
        .description("I'll send a message every 24 hours. Users can click the button to check in!")
        .color(colors::SUCCESS)
        .timestamp(serenity::Timestamp::now());
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Stop the AYDY check in this channel
#[poise::command(prefix_command, slash_command)]
async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();
    let db = KV_DATABASE.get().unwrap();
    let key = format!("aydy_{}", channel_id);

    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(AYDY)?;
        if table.remove(key.as_str())?.is_none() {
            drop(table);
            drop(tx);
            let embed = serenity::CreateEmbed::new()
                .title("❌ Not Running")
                .description("AYDY is not running in this channel.")
                .color(colors::ERROR)
                .timestamp(serenity::Timestamp::now());
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    }
    tx.commit()?;

    let embed = serenity::CreateEmbed::new()
        .title("✅ AYDY Check Stopped")
        .description("The AYDY check has been stopped in this channel.")
        .color(colors::SUCCESS)
        .timestamp(serenity::Timestamp::now());
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Check the current AYDY status
#[poise::command(prefix_command, slash_command)]
async fn status(ctx: Context<'_>) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();
    let db = KV_DATABASE.get().unwrap();
    let key = format!("aydy_{}", channel_id);

    let tx = db.begin_read()?;
    let table = tx.open_table(AYDY)?;

    if let Some(state_str) = table.get(key.as_str())? {
        let state_json: &str = state_str.value();
        let state: AydyState = serde_json::from_str(state_json)?;

        let mut description = String::new();
        description.push_str(&format!(
            "**Enrolled users:** {}\n",
            state.enrolled_users.len()
        ));

        let non_responders = state.get_non_responders(48);
        description.push_str(&format!(
            "**Non-responders (48h):** {}\n\n",
            non_responders.len()
        ));

        if !state.enrolled_users.is_empty() {
            description.push_str("**Enrolled Users:**\n");
            for user_status in state.enrolled_users.values() {
                description.push_str(&format!(
                    "• {} - Last response: <t:{}:R>\n",
                    user_status.user_name, user_status.last_response
                ));
            }
        }

        let embed = serenity::CreateEmbed::new()
            .title("📊 AYDY Status")
            .description(description)
            .color(colors::INFO)
            .timestamp(serenity::Timestamp::now());
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    } else {
        let embed = serenity::CreateEmbed::new()
            .title("❌ Not Running")
            .description("AYDY is not running in this channel.")
            .color(colors::ERROR)
            .timestamp(serenity::Timestamp::now());
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    }

    Ok(())
}

async fn send_aydy_message(
    http: &Arc<serenity::Http>,
    state: &AydyState,
) -> Result<serenity::Message, Error> {
    let channel_id = serenity::ChannelId::new(state.channel_id);

    let embed = create_aydy_embed(state);

    let button = serenity::CreateButton::new("aydy_check")
        .label("Are you dead yet?")
        .style(serenity::ButtonStyle::Primary);

    let components = vec![serenity::CreateActionRow::Buttons(vec![button])];

    let builder = serenity::CreateMessage::new()
        .embed(embed)
        .components(components);

    let message = channel_id.send_message(http, builder).await?;

    Ok(message)
}

async fn update_aydy_message(http: &Arc<serenity::Http>, state: &AydyState) -> Result<(), Error> {
    if let Some(message_id) = state.message_id {
        let channel_id = serenity::ChannelId::new(state.channel_id);
        let message_id = serenity::MessageId::new(message_id);

        let embed = create_aydy_embed(state);

        let button = serenity::CreateButton::new("aydy_check")
            .label("Are you dead yet?")
            .style(serenity::ButtonStyle::Primary);

        let components = vec![serenity::CreateActionRow::Buttons(vec![button])];

        let builder = serenity::EditMessage::new()
            .embed(embed)
            .components(components);

        channel_id.edit_message(http, message_id, builder).await?;
    }

    Ok(())
}

fn create_aydy_embed(state: &AydyState) -> serenity::CreateEmbed {
    let mut description = String::from("Click the button below to let us know you're alive!\n\n");

    // Add enrolled users list
    if !state.enrolled_users.is_empty() {
        description.push_str("**Enrolled Users:**\n");
        for user_status in state.enrolled_users.values() {
            description.push_str(&format!("• {}\n", user_status.user_name));
        }
        description.push('\n');
    }

    // Add non-responders (48 hours)
    let non_responders = state.get_non_responders(48);
    description.push_str("**⚠️ No response in 48 hours:**\n");
    if non_responders.is_empty() {
        description.push_str("• None\n");
    } else {
        for (_, user_status) in &non_responders {
            description.push_str(&format!(
                "• {} (last seen: <t:{}:R>)\n",
                user_status.user_name, user_status.last_response
            ));
        }
    }

    serenity::CreateEmbed::new()
        .title("🩺 Are you dead yet?")
        .description(description)
        .color(colors::INFO)
        .timestamp(serenity::Timestamp::now())
}

pub async fn handle_aydy_button(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    let channel_id = interaction.channel_id.get();
    let user_id = interaction.user.id.get();
    let user_name = interaction.user.name.clone();

    let db = KV_DATABASE.get().unwrap();
    let key = format!("aydy_{}", channel_id);

    // Load state
    let mut state: AydyState = {
        let tx = db.begin_read()?;
        let table: redb::ReadOnlyTable<&str, &str> = tx.open_table(AYDY)?;

        if let Some(state_str) = table.get(key.as_str())? {
            let state_json: &str = state_str.value();
            serde_json::from_str(state_json)?
        } else {
            // AYDY not running in this channel
            interaction
                .create_response(
                    ctx,
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content("❌ AYDY is not running in this channel anymore.")
                            .ephemeral(true),
                    ),
                )
                .await?;
            return Ok(());
        }
    };

    // Update user response
    let is_new = !state.enrolled_users.contains_key(&user_id);
    state.update_user_response(user_id, user_name.clone());

    // Save updated state
    {
        let tx = db.begin_write()?;
        {
            let mut table = tx.open_table(AYDY)?;
            let value = serde_json::to_string(&state)?;
            table.insert(key.as_str(), value.as_str())?;
        }
        tx.commit()?;
    }

    // Update the message
    update_aydy_message(&ctx.http, &state).await?;

    // Respond to the interaction
    let response_msg = if is_new {
        format!(
            "✅ Welcome {}! You've been enrolled in the AYDY check.",
            user_name
        )
    } else {
        format!("✅ Thanks for checking in, {}!", user_name)
    };

    interaction
        .create_response(
            ctx,
            serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content(response_msg)
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}

pub async fn start_aydy_checker(http: Arc<serenity::Http>) {
    let mut interval = interval(Duration::from_secs(3600)); // Check every hour

    log::info!("AYDY checker started");

    loop {
        interval.tick().await;

        if let Err(e) = check_and_send_aydy_messages(&http).await {
            log::error!("Error in AYDY checker: {:?}", e);
        }
    }
}

async fn check_and_send_aydy_messages(http: &Arc<serenity::Http>) -> Result<(), Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_read()?;
    let table = tx.open_table(AYDY)?;

    let mut updates = Vec::new();

    // Collect all AYDY states that need updates
    for entry in table.iter()? {
        let (key, value): (redb::AccessGuard<&str>, redb::AccessGuard<&str>) = entry?;
        let state_json: &str = value.value();
        let mut state: AydyState = serde_json::from_str(state_json)?;

        let now = chrono::Utc::now().timestamp();
        let hours_since_last = (now - state.last_sent) / 3600;

        // Send message every 24 hours
        if hours_since_last >= 24 {
            // Send new message
            if let Ok(message) = send_aydy_message(http, &state).await {
                state.message_id = Some(message.id.get());
                state.last_sent = now;
                updates.push((key.value().to_string(), state));
            }
        }
    }

    drop(table);
    drop(tx);

    // Save updates
    if !updates.is_empty() {
        let tx = db.begin_write()?;
        {
            let mut table = tx.open_table(AYDY)?;
            for (key, state) in updates {
                let value = serde_json::to_string(&state)?;
                table.insert(key.as_str(), value.as_str())?;
            }
        }
        tx.commit()?;
    }

    Ok(())
}
