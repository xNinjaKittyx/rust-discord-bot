use crate::env::FOOTER_URL;
use crate::{Context, Error, HTTP_CLIENT};

use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};
use songbird::input::{Compose, YoutubeDl};

use poise::serenity_prelude as serenity;

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("join", "leave", "queue"),
    subcommand_required
)]
pub async fn music(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

struct TrackErrorNotifier;

#[serenity::async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                log::error!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }

        None
    }
}

#[poise::command(prefix_command, slash_command)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let (guild_id, channel_id) = {
        let guild = ctx.guild().unwrap();
        let channel_id = guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|voice_state| voice_state.channel_id);

        (guild.id, channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            ctx.reply("Not in a voice channel").await?;

            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
    }

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .description("Joined")
            .footer(footer)
            .timestamp(serenity::model::Timestamp::now());
        poise::CreateReply::default().embed(embed)
    };

    ctx.send(reply).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild().unwrap().id;

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let description: String;
    if let Some(handler_lock) = manager.get(guild_id) {
        if let Err(e) = manager.remove(guild_id).await {
            description = format!("Failed to leave {:?}", e)
        } else {
            let handler = handler_lock.lock().await;
            let queue = handler.queue();
            queue.stop();
            description = "Left voice channel and cleared the queue".to_string()
        }
    } else {
        description = "Not in a voice channel".to_string()
    }

    let reply = {
        let embed = serenity::CreateEmbed::new()
            .description(description)
            .footer(footer)
            .timestamp(serenity::model::Timestamp::now());
        poise::CreateReply::default().embed(embed)
    };

    ctx.send(reply).await?;
    Ok(())
}

// #[poise::command(prefix_command, slash_command)]
// pub async fn skip(ctx: Context<'_>, url: String) -> Result<(), Error> {

//     let manager = songbird::get(ctx.serenity_context())
//         .await
//         .expect("Songbird Voice client placed in at initialisation.")
//         .clone();

//     let guild_id = ctx.guild_id().unwrap();
//     if let Some(handler_lock) = manager.get(guild_id) {
//         let mut handler = handler_lock.lock().await;

//         let _ = handler.queue().skip();
//         let current = handler.queue().current();

//         let reply = match current {
//             Some(t) => {
//                 let embed = serenity::CreateEmbed::new()
//                     .title(metadata.title.unwrap_or("Untitled".to_string()))
//                     .description(format!("[Source]({}) - Sample Rate: {} - Duration: {}:{}", metadata.source_url.unwrap_or_default(), metadata.sample_rate.unwrap_or_default(), minutes, seconds))
//                     .image(metadata.thumbnail.unwrap_or_default())
//                     .footer(footer)
//                     .timestamp(serenity::model::Timestamp::now());
//                 poise::CreateReply::default().embed(embed)

//             }
//             None => {

//             }
//         };

//         reply = {
//         };
//     }

//     Ok(())

// }

#[poise::command(prefix_command, slash_command)]
pub async fn queue(ctx: Context<'_>, url: String) -> Result<(), Error> {
    let do_search = !url.starts_with("http");
    let http_client = HTTP_CLIENT.get().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply: poise::reply::CreateReply;
    let guild_id = ctx.guild_id().unwrap();
    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let mut src = if do_search {
            YoutubeDl::new_search(http_client.clone(), url)
        } else {
            YoutubeDl::new(http_client.clone(), url)
        };
        let _ = handler.enqueue_input(src.clone().into()).await;
        let metadata = src.aux_metadata().await?;

        let seconds = metadata.duration.unwrap_or_default().as_secs();
        let minutes = seconds / 60;
        let seconds = seconds % 60;

        reply = {
            let embed = serenity::CreateEmbed::new()
                .title(metadata.title.unwrap_or("Untitled".to_string()))
                .description(format!(
                    "[Source]({}) - Sample Rate: {} - Duration: {}:{}",
                    metadata.source_url.unwrap_or_default(),
                    metadata.sample_rate.unwrap_or_default(),
                    minutes,
                    seconds
                ))
                // .image(metadata.thumbnail.unwrap_or_default())
                .footer(footer)
                .timestamp(serenity::model::Timestamp::now());
            poise::CreateReply::default().embed(embed)
        };
    } else {
        reply = {
            let embed = serenity::CreateEmbed::new()
                .description("I'm not in a channel.")
                .footer(footer)
                .timestamp(serenity::model::Timestamp::now());
            poise::CreateReply::default().embed(embed)
        };
    }
    ctx.send(reply).await?;

    Ok(())
}
