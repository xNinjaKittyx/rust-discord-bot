use poise::serenity_prelude as serenity;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Deserialize, Debug)]
pub struct ReactionConfig {
    pub enabled: bool,
    pub animated: Option<bool>,
    pub emoji_id: Option<u64>,
    pub emoji_name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ReplyConfig {
    pub enabled: bool,
    pub text: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    pub contains: Vec<String>,
    pub case_sensitive: bool,
    pub exact_match: bool,
    pub author: Option<Vec<serenity::UserId>>,
    pub reaction: Option<ReactionConfig>,
    pub reply: Option<ReplyConfig>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub response: HashMap<String, Response>,
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_content)?;
    Ok(config)
}

pub async fn handle_message_response(
    ctx: &serenity::Context,
    new_message: &serenity::Message,
    config: &Config,
) -> Result<(), Error> {
    for (_, response) in &config.response {
        if let Some(authors) = &response.author {
            if !authors.is_empty() && !authors.contains(&new_message.author.id) {
                continue;
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

    Ok(())
}
