
use poise::serenity_prelude as serenity;
use url::Url;


pub async fn check_twitter_embed(
    ctx: &serenity::Context,
    message: &serenity::Message,
) -> Result<(), serenity::Error> {

    let url_start = message.content.find("https://twitter.com/")
        .or_else(|| message.content.find("https://x.com/"));

    if let Some(start) = url_start {
        let url_str = &message.content[start..];
        let url_end = url_str.find(char::is_whitespace).unwrap_or(url_str.len());
        let url = &url_str[..url_end];

        if let Ok(parsed_url) = Url::parse(url) {
            let path = parsed_url.path();
            let fixed_url = format!("https://fixupx.com{}", path);

            message.reply(
                ctx, format!("Detected a Twitter/X link, here's a fixed embed: {}", fixed_url)
            ).await?;
        }
    }
    Ok(())
}
