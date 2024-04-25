use crate::{Context, Error};

#[poise::command(prefix_command, slash_command)]
pub async fn anime(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("TBD ANIME! XD").await?;
    Ok(())
}
