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

use poise::serenity_prelude as serenity;

use songbird::SerenityInit;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
pub struct Data {}

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static KV_DATABASE: OnceLock<redb::Database> = OnceLock::new();

const TABLE: redb::TableDefinition<&str, &str> = redb::TableDefinition::new("tags");

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
                let response = localai::get_gpt_response(new_message, &ctx).await?;
                new_message
                    .reply(ctx, format!("{}", response.choices[0].message.content))
                    .await?;
            }
            // If someone
            else if new_message.content.to_lowercase().contains("fuck")
                && new_message.author.id != ctx.cache.current_user().id
            {
                new_message.reply(ctx, format!("Fuck you too")).await?;
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

#[tokio::main]
async fn main() {
    env_logger::init();

    KV_DATABASE.get_or_init(|| redb::Database::create("storage.db").unwrap());

    HTTP_CLIENT.get_or_init(|| reqwest::Client::new());

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                anime::anime(),
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

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .activity(activity)
        .register_songbird()
        .await;

    client.unwrap().start().await.unwrap()
}
