use crate::env::{FOOTER_URL, LOCALAI_URL, SERVE_STATIC_URL};
use crate::{Context, Error, HTTP_CLIENT};

use std::cmp;

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

fn stablediffusion_help() -> String {
    String::from("Generate an image with Dreamshaper")
}

#[poise::command(prefix_command, slash_command, help_text_fn = "stablediffusion_help", category = "AI")]
pub async fn stablediffusion(
    ctx: Context<'_>,
    #[description = "Use a | to split between positive and negative attributes in your prompt."]
    prompt: String,
    width: Option<u16>,
    height: Option<u16>,
) -> Result<(), Error> {
    log::info!("Generating Stable Diffusion with {}", prompt);

    let model_name = "dreamshaper"; // TODO: Configurable

    let map = SDPrompt {
        prompt: prompt.to_string(),
        model: model_name.to_string(),
        step: 15,
        size: format!(
            "{}x{}",
            cmp::min(width.unwrap_or(1920), 3840),
            cmp::min(height.unwrap_or(1080), 2160),
        ),
    };

    let resp = HTTP_CLIENT
        .get()
        .unwrap()
        .post(format!("{}/v1/images/generations", &*LOCALAI_URL))
        .header("Content-Type", "application/json")
        .json(&map)
        .send()
        .await?;

    let json_string = resp.text().await?;
    log::info!("{}", json_string);
    let results: Result<SDPromptResponse, serde_json::Error> =
        serde_json::from_str(json_string.as_str());
    let response = results.unwrap();

    let footer = serenity::CreateEmbedFooter::new(format!("Powered by {}", &*FOOTER_URL));
    let reply = {
        let embed = serenity::CreateEmbed::new()
            .title(format!("StableDiffusion with {}", model_name))
            .description(prompt)
            .image(format!(
                "{}{}",
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
