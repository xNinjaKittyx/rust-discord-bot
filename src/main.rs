#![feature(lazy_cell)]

mod anime;
mod basic;
mod kanji;
mod sd;
mod sonarr;
mod sonarr_serde;

use std::sync::LazyLock;
use std::sync::OnceLock;

use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
// User data, which is stored and accessible in all command invocations
pub struct Data {}

pub static FOOTER_URL: LazyLock<String> = LazyLock::new(|| std::env::var("FOOTER_URL").unwrap());

pub static LOCALAI_URL: LazyLock<String> = LazyLock::new(|| std::env::var("LOCALAI_URL").unwrap());

pub static SERVE_STATIC_URL: LazyLock<String> =
    LazyLock::new(|| std::env::var("SERVE_STATIC_URL").unwrap());

#[derive(Debug, Serialize, Deserialize)]
struct ModelMessageData {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelData {
    pub model: String,
    pub temperature: f32,
    pub messages: Vec<ModelMessageData>,
}

impl Default for ModelMessageData {
    fn default() -> Self {
        ModelMessageData {
            role: "system".to_string(),
            content: "Given the following conversation, relevant context, and a follow up question, reply with an answer to the current question the user is asking. Return only your response to the question given the above information following the users instructions as needed.".to_string()
        }
    }
}

impl ModelData {
    fn new() -> Self {
        Default::default()
    }
}

impl Default for ModelData {
    fn default() -> Self {
        ModelData {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            messages: vec![
                ModelMessageData {
                    ..Default::default()
                },
                ModelMessageData {
                    ..Default::default()
                },
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelResponseMessage {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelResponseChoice {
    pub message: ModelResponseMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelResponse {
    pub choices: Vec<ModelResponseChoice>,
}

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

// Event Handler
async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            if new_message.mentions.contains(&**ctx.cache.current_user())
                && new_message.author.id != ctx.cache.current_user().id
            {
                let mut map = ModelData::new();
                map.messages[1].role = "user".to_string();
                map.messages[1].content = format!("{}", new_message.content);
                let resp = HTTP_CLIENT
                    .get_or_init(|| reqwest::Client::new())
                    .post(format!("{}/v1/chat/completions", &*LOCALAI_URL))
                    .json(&map)
                    .send()
                    .await?;

                let json_string = resp.text().await?;

                println!("{}", json_string);

                // Deserialize the JSON string into a Value
                let results: Result<ModelResponse, serde_json::Error> =
                    serde_json::from_str(json_string.as_str());
                let response = results.unwrap();

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

    HTTP_CLIENT.get_or_init(|| reqwest::Client::new());

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                anime::anime(),
                basic::ping(),
                basic::reply(),
                basic::help(),
                kanji::kanji(),
                sd::stablediffusion(),
                sonarr::sonarr(),
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            on_error: |error| {
                Box::pin(async move {
                    println!("what the hell");
                    match error {
                        poise::FrameworkError::ArgumentParse { error, .. } => {
                            if let Some(error) = error.downcast_ref::<serenity::RoleParseError>() {
                                println!("Found a RoleParseError: {:?}", error);
                            } else {
                                println!("Not a RoleParseError :(");
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

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap()
}
// // This trait adds the `register_songbird` and `register_songbird_with` methods
// // to the client builder below, making it easy to install this voice client.
// // The voice client can be retrieved in any command using `songbird::get(ctx).await`.
// use songbird::SerenityInit;

// use serenity::{
//     async_trait,
//     client::{Client, EventHandler},
//     framework::{
//         StandardFramework,
//         standard::{
//             Args, CommandResult,
//             macros::{command, group, hook},
//         },
//     },
//     model::{channel::Message, gateway::Ready, event::ResumedEvent},
//     prelude::GatewayIntents,
//     Result as SerenityResult,
// };
// use tracing::{debug, error, info, instrument};

// #[group]
// #[commands(deafen, join, leave, mute, play, ping, undeafen, unmute, sonarr)]
// struct General;

// struct Handler;

// #[async_trait]
// impl EventHandler for Handler {
//     async fn ready(&self, _: Context, ready: Ready) {
//         // Log at the INFO level. This is a macro from the `tracing` crate.
//         info!("{} is connected!", ready.user.name);
//     }

//     // For instrument to work, all parameters must implement Debug.
//     //
//     // Handler doesn't implement Debug here, so we specify to skip that argument.
//     // Context doesn't implement Debug either, so it is also skipped.
//     #[instrument(skip(self, _ctx))]
//     async fn resume(&self, _ctx: Context, resume: ResumedEvent) {
//         // Log at the DEBUG level.
//         //
//         // In this example, this will not show up in the logs because DEBUG is
//         // below INFO, which is the set debug level.
//         debug!("Resumed; trace: {:?}", resume.trace);
//     }

// }

// // instrument will show additional information on all the logs that happen inside
// // the function.
// //
// // This additional information includes the function name, along with all it's arguments
// // formatted with the Debug impl.
// // This additional information will also only be shown if the LOG level is set to `debug`
// #[hook]
// #[instrument]
// async fn before(_: &Context, msg: &Message, command_name: &str) -> bool {
//     info!("Got command '{}' by user '{}'", command_name, msg.author.name);

//     true
// }

// #[tokio::main]
// async fn main() {
//     // Call tracing_subscriber's initialize function, which configures `tracing`
//     // via environment variables.
//     //
//     // For example, you can say to log all levels INFO and up via setting the
//     // environment variable `RUST_LOG` to `INFO`.
//     //
//     // This environment variable is already preset if you use cargo-make to run
//     // the example.
//     tracing_subscriber::fmt::init();

//     // Login with a bot token from the environment
//     let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment.");

//     let framework = StandardFramework::new()
//         .configure(|c| c.prefix("--")) // set the bot's prefix to "~"
//         .group(&GENERAL_GROUP);

//     let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
//     let mut client = Client::builder(token, intents)
//         .event_handler(Handler)
//         .framework(framework)
//         .register_songbird()
//         .await
//         .expect("Error creating client");

//     tokio::spawn(async move {
//         let _ = client.start().await.map_err(|why| println!("Client ended: {:?}", why));
//     });

//     tokio::signal::ctrl_c().await;
//     println!("Received Ctrl-C, shutting down.");
// }

// #[command]
// #[only_in(guilds)]
// async fn deafen(ctx: &Context, msg: &Message) -> CommandResult {
//     let guild = msg.guild(&ctx.cache).unwrap();
//     let guild_id = guild.id;
//     let manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialization.").clone();

//     let handler_lock = match manager.get(guild_id) {
//         Some(handler) => handler,
//         None => {
//             check_msg(msg.reply(ctx, "Not in a voice channel").await);

//             return Ok(());
//         },
//     };
//     let mut handler = handler_lock.lock().await;

//     if handler.is_deaf() {
//         check_msg(msg.channel_id.say(&ctx.http, "Already deafened").await);
//     } else {
//         if let Err(e) = handler.deafen(true).await {
//             check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
//         }

//         check_msg(msg.channel_id.say(&ctx.http, "Deafened").await);
//     }

//     Ok(())
// }
// #[command]
// #[only_in(guilds)]
// async fn join(ctx: &Context, msg: &Message) -> CommandResult {
//     let guild = msg.guild(&ctx.cache).unwrap();
//     let guild_id = guild.id;

//     let channel_id = guild
//         .voice_states.get(&msg.author.id)
//         .and_then(|voice_state| voice_state.channel_id);

//     let connect_to = match channel_id {
//         Some(channel) => channel,
//         None => {
//             check_msg(msg.reply(ctx, "Not in a voice channel").await);

//             return Ok(());
//         }
//     };

//     let manager = songbird::get(ctx).await
//         .expect("Songbird Voice client placed in at initialisation.").clone();

//     let _handler = manager.join(guild_id, connect_to).await;
//     check_msg(msg.channel_id.say(&ctx.http, format!("Joined voice chat: {:?}", msg.channel_id)).await);

//     Ok(())
// }

// #[command]
// #[only_in(guilds)]
// async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
//     let guild = msg.guild(&ctx.cache).unwrap();
//     let guild_id = guild.id;

//     let manager = songbird::get(ctx).await
//         .expect("Songbird Voice client placed in at initialization.").clone();
//     let has_handler = manager.get(guild_id).is_some();

//     if has_handler {
//         if let Err(e) = manager.remove(guild_id).await {
//             check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
//         }

//         check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
//     } else {
//         check_msg(msg.reply(ctx, "Not in a voice channel").await);
//     }

//     Ok(())
// }

// #[command]
// #[only_in(guilds)]
// async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
//     if let Err(why) = msg.channel_id.say(&ctx.http, "Pong! : )").await {
//         error!("Error sending message: {:?}", why);
//     }

//     Ok(())
// }

// #[command]
// #[only_in(guilds)]
// async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
//     let url = match args.single::<String>() {
//         Ok(url) => url,
//         Err(_) => {
//             check_msg(msg.channel_id.say(&ctx.http, "Must provide a URL to a video or audio").await);

//             return Ok(());
//         },
//     };

//     if !url.starts_with("http") {
//         check_msg(msg.channel_id.say(&ctx.http, "Must provide a valid URL").await);

//         return Ok(());
//     }

//     let guild = msg.guild(&ctx.cache).unwrap();
//     let guild_id = guild.id;

//     let manager = songbird::get(ctx).await
//         .expect("Songbird Voice client placed in at initialisation.").clone();

//     if let Some(handler_lock) = manager.get(guild_id) {
//         let mut handler = handler_lock.lock().await;

//         let source = match songbird::ytdl(&url).await {
//             Ok(source) => source,
//             Err(why) => {
//                 println!("Err starting source: {:?}", why);

//                 check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

//                 return Ok(());
//             },
//         };

//         handler.play_source(source);

//         check_msg(msg.channel_id.say(&ctx.http, "Playing song").await);
//     } else {
//         check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to play in").await);
//     }

//     Ok(())
// }

// #[command]
// #[only_in(guilds)]
// async fn undeafen(ctx: &Context, msg: &Message) -> CommandResult {
//     if let Err(why) = msg.channel_id.say(&ctx.http, "Pong! : )").await {
//         error!("Error sending message: {:?}", why);
//     }

//     Ok(())
// }

// #[command]
// #[only_in(guilds)]
// async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {

//     if let Err(why) = msg.channel_id.say(&ctx.http, "Pong! : )").await {
//         error!("Error sending message: {:?}", why);
//     }

//     Ok(())
// }
