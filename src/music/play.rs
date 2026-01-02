use std::sync::Arc;
use std::time::Duration;

use crate::env::FOOTER_URL;
use crate::{Context, Error, HTTP_CLIENT};

use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent};
use songbird::input::{Compose, YoutubeDl};
use songbird::tracks::PlayMode;

use poise::serenity_prelude as serenity;

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("join", "leave", "queue"),
    subcommand_required,
    category = "Music"
)]
pub async fn music(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

struct TrackErrorNotifier;
struct TrackStartNotifier {
    pub ctx: serenity::Context,
    pub http: Arc<serenity::Http>,
    pub channel_id: serenity::ChannelId,
    pub embed: serenity::CreateEmbed,
}

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

#[serenity::async_trait]
impl VoiceEventHandler for TrackStartNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        log::info!("{:?}", ctx);
        let ctx_id = self.channel_id.get();
        let play_button_id = format!("{}play", ctx_id);
        let pause_button_id = format!("{}pause", ctx_id);
        let next_button_id = format!("{}next", ctx_id);
        let volume_down_id = format!("{}voldown", ctx_id);
        let volume_up_id = format!("{}volup", ctx_id);

        if let EventContext::Track([(state, track)]) = ctx {
            while state.playing != PlayMode::Play {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }

            let _ = self
                .channel_id
                .send_message(
                    &self.ctx,
                    serenity::CreateMessage::new()
                        .add_embed(
                            self.embed
                                .clone()
                                .description(format!("Playing Now ---- Volume: {}", state.volume)),
                        )
                        .components(vec![serenity::CreateActionRow::Buttons(vec![
                            serenity::CreateButton::new(&play_button_id).emoji('▶'),
                            serenity::CreateButton::new(&pause_button_id).emoji('⏸'),
                            serenity::CreateButton::new(&next_button_id).emoji('⏭'),
                            serenity::CreateButton::new(&volume_down_id).emoji('🔉'),
                            serenity::CreateButton::new(&volume_up_id).emoji('🔊'),
                        ])]),
                )
                .await
                .unwrap();
            log::info!("STARTED PLAYING :))))");
            while let Some(press) =
                serenity::collector::ComponentInteractionCollector::new(&self.ctx)
                    .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
                    .timeout(std::time::Duration::from_secs(3600 * 24))
                    .await
            {
                let new_state = track.get_info().await.unwrap();
                if new_state.playing == PlayMode::End {
                    break;
                }
                if press.data.custom_id == play_button_id {
                    let _ = track.play();
                } else if press.data.custom_id == pause_button_id {
                    let _ = track.pause();
                } else if press.data.custom_id == volume_down_id {
                    let _ = track.set_volume((new_state.volume - 0.1).max(0.0));
                } else if press.data.custom_id == volume_up_id {
                    let _ = track.set_volume((new_state.volume + 0.1).min(2.0));
                } else if press.data.custom_id == next_button_id {
                    let _ = track.stop();
                    let _ = press
                        .create_response(
                            &self.ctx,
                            serenity::CreateInteractionResponse::UpdateMessage(
                                serenity::CreateInteractionResponseMessage::new()
                                    .embed(self.embed.clone().description("Ended"))
                                    .components(Vec::new()),
                            ),
                        )
                        .await;
                    break;
                }
                let _ = press
                    .create_response(
                        &self.ctx,
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::new().embed(
                                self.embed.clone().description(format!(
                                    "Playing Now ---- Volume: {}",
                                    new_state.volume
                                )),
                            ),
                        ),
                    )
                    .await;
            }
        }
        None
    }
}

#[poise::command(prefix_command, slash_command, category = "Music")]
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

#[poise::command(prefix_command, slash_command, category = "Music")]
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

#[poise::command(prefix_command, slash_command, category = "Music")]
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
        let metadata = src.aux_metadata().await?;

        let seconds = metadata.duration.unwrap_or_default().as_secs();
        let minutes = seconds / 60;
        let seconds = seconds % 60;
        let title = metadata.title.unwrap_or("Untitled".to_string());
        let http = ctx.serenity_context().http.clone();

        let embed = serenity::CreateEmbed::new()
            .title(title)
            .description("Queued")
            .field(
                "Youtube Video",
                format!("[Source]({})", metadata.source_url.unwrap_or_default()),
                true,
            )
            .field(
                "Sample Rate",
                metadata.sample_rate.unwrap_or_default().to_string(),
                true,
            )
            .field("Duration", format!("{}:{}", minutes, seconds), true)
            .footer(footer)
            .timestamp(serenity::model::Timestamp::now());

        let played_embed = embed.clone().image(metadata.thumbnail.unwrap());

        let track_handle = handler.enqueue_input(src.clone().into()).await;
        let _ = track_handle.pause();
        let _ = track_handle.add_event(
            Event::Track(TrackEvent::Playable),
            TrackStartNotifier {
                ctx: ctx.serenity_context().clone(),
                http,
                channel_id: ctx.channel_id(),
                embed: played_embed,
            },
        );
        let _ = match handler.queue().current() {
            Some(track_handle) => track_handle.play(),
            None => Ok(()),
        };

        reply = poise::CreateReply::default().embed(embed);
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
