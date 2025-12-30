use crate::env::FOOTER_URL;
use crate::{Context, Error, KV_DATABASE, TABLE};

use poise::serenity_prelude as serenity;
use redb::ReadableTable;

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("add", "remove", "show", "showall"),
    subcommand_required,
    category = "Utility"
)]
pub async fn tag(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, owners_only, category = "Utility")]
pub async fn add(ctx: Context<'_>, key: String, value: String) -> Result<(), Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(TABLE)?;
        let _ = table.insert(key.as_str(), value.as_str());
    }
    tx.commit()?;

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title("Added Tag")
            .fields(vec![(key, value, true)])
            .footer(footer)
            .timestamp(serenity::model::Timestamp::now());

        poise::CreateReply::default().embed(embed)
    };
    ctx.send(reply).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, owners_only, category = "Utility")]
pub async fn remove(ctx: Context<'_>, key: String) -> Result<(), Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(TABLE)?;
        let _ = table.remove(key.as_str());
    }
    tx.commit()?;

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title("Removed Tag")
            .description(key)
            .footer(footer)
            .timestamp(serenity::model::Timestamp::now());

        poise::CreateReply::default().embed(embed)
    };
    ctx.send(reply).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Utility")]
pub async fn show(ctx: Context<'_>, key: String) -> Result<(), Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_read()?;
    let value = {
        let table = tx.open_table(TABLE)?;
        table.get(key.as_str())?
    };

    let desc = match value {
        Some(value) => value.value().to_owned(),
        None => "".to_string(),
    };

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title(key)
            .description(desc)
            .footer(footer)
            .timestamp(serenity::model::Timestamp::now());

        poise::CreateReply::default().embed(embed)
    };
    ctx.send(reply).await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "Utility")]
pub async fn showall(ctx: Context<'_>) -> Result<(), Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_read()?;
    let table = tx.open_table(TABLE)?;
    let mut description = String::new();
    for value in table.iter()? {
        description.push_str(format!("{}\n", value.unwrap().0.value()).as_str());
    }

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title("")
            .description(description)
            .footer(footer)
            .timestamp(serenity::model::Timestamp::now());

        poise::CreateReply::default().embed(embed)
    };
    ctx.send(reply).await?;

    Ok(())
}
