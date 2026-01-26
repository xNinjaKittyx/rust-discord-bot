use poise::serenity_prelude as serenity;

use crate::config::Config;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn handle_message_response(
    ctx: &serenity::Context,
    new_message: &serenity::Message,
    config: &Config,
) -> Result<(), Error> {
    for response in config.response.values() {
        if let Some(authors) = &response.author
            && !authors.is_empty()
            && !authors.contains(&new_message.author.id)
        {
            continue;
        }

        let message_content = if response.case_sensitive {
            new_message.content.clone()
        } else {
            new_message.content.to_lowercase()
        };

        let matches = if response.exact_match {
            response.contains.contains(&message_content)
        } else {
            response
                .contains
                .iter()
                .any(|c| message_content.contains(c))
        };

        if matches {
            if let Some(reaction_config) = &response.reaction
                && reaction_config.enabled
                && let (Some(animated), Some(emoji_id), Some(emoji_name)) = (
                    reaction_config.animated,
                    reaction_config.emoji_id,
                    &reaction_config.emoji_name,
                )
            {
                let reaction = serenity::ReactionType::Custom {
                    animated,
                    id: serenity::EmojiId::new(emoji_id),
                    name: Some(emoji_name.clone()),
                };
                new_message.react(ctx, reaction).await?;
            }

            if let Some(reply_config) = &response.reply
                && reply_config.enabled
                && let Some(text) = &reply_config.text
            {
                new_message.reply(ctx, text).await?;
            }
        }
    }

    Ok(())
}
