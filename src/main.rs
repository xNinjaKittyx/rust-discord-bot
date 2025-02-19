#![feature(lazy_cell)]

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

use std::sync::OnceLock;

use miette::Result;
use poise::serenity_prelude as serenity;
use songbird::SerenityInit;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
pub struct Data {}

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static KV_DATABASE: OnceLock<redb::Database> = OnceLock::new();

const TABLE: redb::TableDefinition<&str, &str> = redb::TableDefinition::new("tags");
const AI_CONTEXT: redb::TableDefinition<&str, &str> = redb::TableDefinition::new("context");

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
        serenity::FullEvent::Message { new_message } => {
            if new_message.mentions.contains(&**ctx.cache.current_user())
                && new_message.author.id != ctx.cache.current_user().id
            {
                if new_message
                    .content
                    .to_lowercase()
                    .contains("please wipe all context")
                    && new_message.author.id == 82221891191844864
                {
                    localai::wipe_context(new_message).await?;
                    new_message
                        .reply(ctx, format!("Wiped all context."))
                        .await?;
                } else {
                    let typing = ctx.http.start_typing(new_message.channel_id);
                    let response = localai::get_gpt_response(new_message, &ctx).await?;
                    new_message.reply(ctx, format!("{}", response)).await?;
                    typing.stop();
                }
            }
            // If someone
            else if new_message.content.to_lowercase().contains("fuck")
                && new_message.author.id != ctx.cache.current_user().id
            {
                let reaction = serenity::ReactionType::Custom {
                    animated: false,
                    id: serenity::EmojiId::new(1106775701832609902),
                    name: Some("xdd".to_string()),
                };
                new_message.react(ctx, reaction).await?;
                // new_message.reply(ctx, format!("Fuck you too")).await?;
            } else if new_message.content.to_lowercase().contains("swipe")
                && new_message.author.id != ctx.cache.current_user().id
            {
                new_message
                    .reply(
                        ctx,
                        format!("https://www.youtube.com/watch?v=juN3vjJAzME&t=35s"),
                    )
                    .await?;
            } else if new_message.content.to_lowercase().contains("boosted")
                && new_message.author.id != ctx.cache.current_user().id
            {
                let reaction = serenity::ReactionType::Custom {
                    animated: false,
                    id: serenity::EmojiId::new(1106775701832609902),
                    name: Some("xdd".to_string()),
                };
                new_message.react(ctx, reaction).await?;
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

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                anime::guess(),
                basic::avatar(),
                basic::ping(),
                basic::help(),
                fun::shibe(),
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
                    log::warn!("Failed with {}", error);
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

    let token = std::env::var("DISCORD_TOKEN").unwrap();
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
