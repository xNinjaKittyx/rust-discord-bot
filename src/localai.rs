use crate::env::LOCALAI_URL;
use crate::{Error, HTTP_CLIENT};

use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ModelMessageData {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelData {
    pub model: String,
    pub temperature: f32,
    pub messages: Vec<ModelMessageData>,
}

impl Default for ModelMessageData {
    fn default() -> Self {
        ModelMessageData {
            role: "system".to_string(),
            content: "Given the following conversation, relevant context, and a follow up question, reply with an answer to the current question the user is asking. Return only your response to the question given the above information following the users instructions as needed.".to_string()
        }
    }
}

impl ModelData {
    fn new() -> Self {
        Default::default()
    }
}

impl Default for ModelData {
    fn default() -> Self {
        ModelData {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            messages: vec![
                ModelMessageData {
                    ..Default::default()
                },
                ModelMessageData {
                    ..Default::default()
                },
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelResponseMessage {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelResponseChoice {
    pub message: ModelResponseMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelResponse {
    pub choices: Vec<ModelResponseChoice>,
}

pub async fn get_gpt_response(message: &serenity::Message) -> Result<ModelResponse, Error> {
    let mut map = ModelData::new();
    map.messages[1].role = "user".to_string();
    map.messages[1].content = format!("{}", message.content);
    let resp = HTTP_CLIENT
        .get_or_init(|| reqwest::Client::new())
        .post(format!("{}/v1/chat/completions", &*LOCALAI_URL))
        .json(&map)
        .send()
        .await?;

    let json_string = resp.text().await?;

    println!("{}", json_string);

    // Deserialize the JSON string into a Value
    let results: Result<ModelResponse, serde_json::Error> =
        serde_json::from_str(json_string.as_str());
    Ok(results.unwrap())
}
