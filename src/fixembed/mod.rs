
pub mod insta;
pub mod reddit;
pub mod twitter;

use poise::serenity_prelude as serenity;


pub async fn process_fix_embed(
    ctx: &serenity::Context,
    message: &serenity::Message,
) -> Result<(), serenity::Error> {

    twitter::check_twitter_embed(ctx, message).await?;
    insta::check_instagram_embed(ctx, message).await?;
    reddit::check_reddit_embed(ctx, message).await?;
    Ok(())
}
