mod anime;
mod basic;
mod env;
mod fun;
mod kanji;
mod localai;
mod music;
mod sd;
mod shoko;
mod sonarr;
mod sonarr_serde;
mod tags;

use std::fs;
use std::sync::OnceLock;
use std::time::Instant;

use ::serenity::all::{MessageBuilder, UserId};
use miette::Result;
use poise::serenity_prelude as serenity;
use songbird::SerenityInit;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
pub struct Data {}

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static KV_DATABASE: OnceLock<redb::Database> = OnceLock::new();
static REACTION_CONFIG: OnceLock<Config> = OnceLock::new();

const TABLE: redb::TableDefinition<&str, &str> = redb::TableDefinition::new("tags");
const AI_CONTEXT: redb::TableDefinition<&str, &str> = redb::TableDefinition::new("context");

fn split_string_chunks(long_string: &str, chunk_size: usize) -> Vec<String> {
    long_string
        .chars()
        .collect::<Vec<char>>() // Convert to Vec<char> to use chunks()
        .chunks(chunk_size)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct ReactionConfig {
    enabled: bool,
    animated: Option<bool>,
    emoji_id: Option<u64>,
    emoji_name: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ReplyConfig {
    enabled: bool,
    text: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Response {
    contains: Vec<String>,
    case_sensitive: bool,
    exact_match: bool,
    author: Option<Vec<serenity::UserId>>, // Add author field
    reaction: Option<ReactionConfig>,
    reply: Option<ReplyConfig>,
}

#[derive(Deserialize, Debug)]
struct Config {
    response: std::collections::HashMap<String, Response>,
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_content)?;
    Ok(config)
}

// Event Handler
async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            log::info!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => 'message_match: {
            let lower_case_msg = new_message.content.to_lowercase();
            if new_message.author.id == ctx.cache.current_user().id {
                break 'message_match;
            }
            if new_message.mentions.contains(&**ctx.cache.current_user()) {
                if lower_case_msg.contains("please wipe all context")
                    && new_message.author.id == *env::AUTHOR_ID
                {
                    localai::wipe_context(new_message).await?;
                    new_message
                        .reply(ctx, format!("Wiped all context."))
                        .await?;
                } else {
                    let start = Instant::now();
                    let typing = ctx.http.start_typing(new_message.channel_id);
                    let response = localai::get_gpt_response(new_message, &ctx).await?;

                    log::info!("Response: {:#?}", response);
                    let chunks = split_string_chunks(&response, 2000);
                    log::info!("Chunks: {:#?}", chunks);
                    for chunk in chunks.iter() {
                        new_message.reply(ctx, format!("{}", chunk)).await?;
                    }

                    typing.stop();
                    let duration = start.elapsed();
                    println!("Time elapsed for GPT Response is: {:?}", duration);
                }
            }

            let config = REACTION_CONFIG.get().unwrap();

            for (_, response) in &config.response {
                if let Some(authors) = &response.author {
                    if !authors.is_empty() && !authors.contains(&new_message.author.id) {
                        continue; // Skip if the author is not in the list
                    }
                }

                let message_content = if response.case_sensitive {
                    new_message.content.clone()
                } else {
                    new_message.content.to_lowercase()
                };

                let matches = if response.exact_match {
                    response.contains.iter().any(|c| message_content == *c)
                } else {
                    response
                        .contains
                        .iter()
                        .any(|c| message_content.contains(c))
                };

                if matches {
                    if let Some(reaction_config) = &response.reaction {
                        if reaction_config.enabled {
                            if let (Some(animated), Some(emoji_id), Some(emoji_name)) = (
                                reaction_config.animated,
                                reaction_config.emoji_id,
                                &reaction_config.emoji_name,
                            ) {
                                let reaction = serenity::ReactionType::Custom {
                                    animated,
                                    id: serenity::EmojiId::new(emoji_id),
                                    name: Some(emoji_name.clone()),
                                };
                                new_message.react(ctx, reaction).await?;
                            }
                        }
                    }

                    if let Some(reply_config) = &response.reply {
                        if reply_config.enabled {
                            if let Some(text) = &reply_config.text {
                                new_message.reply(ctx, text).await?;
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn discordbot(subsys: tokio_graceful_shutdown::SubsystemHandle) -> Result<()> {
    log::info!("Initializing clients and databases...");

    KV_DATABASE.get_or_init(|| redb::Database::create("storage.db").unwrap());

    // Make sure tables exist
    let db = KV_DATABASE.get().unwrap();
    {
        let tx = db.begin_write().unwrap();
        tx.open_table(TABLE).unwrap();
        tx.open_table(AI_CONTEXT).unwrap();
        tx.commit().unwrap();
    }

    HTTP_CLIENT.get_or_init(|| reqwest::Client::new());
    REACTION_CONFIG.get_or_init(|| load_config().unwrap());

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                anime::guess(),
                basic::avatar(),
                basic::ping(),
                basic::help(),
                fun::fox(),
                kanji::kanji(),
                music::music(),
                sd::stablediffusion(),
                shoko::shoko(),
                sonarr::sonarr(),
                tags::tag(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            on_error: |error| {
                Box::pin(async move {
                    log::info!("Failed with {}", error);
                    match error {
                        poise::FrameworkError::ArgumentParse { error, .. } => {
                            if let Some(error) = error.downcast_ref::<serenity::RoleParseError>() {
                                log::warn!("Found a RoleParseError: {:?}", error);
                            } else {
                                log::warn!("Not a RoleParseError :(");
                            }
                        }
                        other => poise::builtins::on_error(other).await.unwrap(),
                    }
                })
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let token = &*env::DISCORD_TOKEN;
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let activity = serenity::ActivityData {
        name: "you".to_string(),
        kind: serenity::ActivityType::Watching,
        state: None,
        url: None,
    };

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .activity(activity)
        .register_songbird()
        .await
        .expect("Error Creating Client");

    log::info!("Starting Discord Client...");
    let _ = client
        .start()
        .await
        .map_err(|why| println!("Client ended: {:?}", why));
    subsys.on_shutdown_requested().await;
    log::info!("Shutting down!");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    log::info!("Starting tokio startup...");

    // Setup and execute subsystem tree
    tokio_graceful_shutdown::Toplevel::new(|s| async move {
        s.start(tokio_graceful_shutdown::SubsystemBuilder::new(
            "Subsys1", discordbot,
        ));
    })
    .catch_signals()
    .handle_shutdown_requests(std::time::Duration::from_millis(1000))
    .await
    .map_err(Into::into)
}
