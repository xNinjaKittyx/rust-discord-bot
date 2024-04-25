use crate::{Context, Error};

#[poise::command(prefix_command, slash_command)]
pub async fn kanji(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("TBD KANJI! XD").await?;
    Ok(())
}
