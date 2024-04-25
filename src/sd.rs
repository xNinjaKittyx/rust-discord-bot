use crate::env::{FOOTER_URL, LOCALAI_URL, SERVE_STATIC_URL};
use crate::{Context, Error, HTTP_CLIENT};

use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct SDPrompt {
    prompt: String,
    model: String,
    step: u32,
    size: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SDPromptResponseObjects {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SDPromptResponse {
    data: Vec<SDPromptResponseObjects>,
}

#[poise::command(prefix_command, slash_command)]
pub async fn stablediffusion(ctx: Context<'_>, prompt: String) -> Result<(), Error> {
    println!("Generating Stable Diffusion with {}", prompt);

    let map = SDPrompt {
        prompt: prompt.to_string(),
        model: "animagine-xl".to_string(),
        step: 51,
        size: "1024x1024".to_string(),
    };

    let resp = HTTP_CLIENT
        .get_or_init(|| reqwest::Client::new())
        .post(format!("{}/v1/images/generations", &*LOCALAI_URL))
        .header("Content-Type", "application/json")
        .json(&map)
        .send()
        .await?;

    let json_string = resp.text().await?;
    println!("{}", json_string);
    let results: Result<SDPromptResponse, serde_json::Error> =
        serde_json::from_str(json_string.as_str());
    let response = results.unwrap();

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title("StableDiffusion with animagine-xl")
            .description(prompt)
            .image(format!(
                "{}/{}",
                &*SERVE_STATIC_URL,
                response.data[0].url.strip_prefix(&*LOCALAI_URL).unwrap()
            ))
            // .fields(fields_vector)
            .footer(footer)
            // Add a timestamp for the current time
            // This also accepts a rfc3339 Timestamp
            .timestamp(serenity::model::Timestamp::now());

        poise::CreateReply::default().embed(embed)
    };

    ctx.send(reply).await?;

    Ok(())
}
