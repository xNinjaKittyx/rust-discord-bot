// https://github.com/ollama/ollama/blob/main/docs/api.md

use std::collections::VecDeque;
use std::sync::mpsc::channel;

use crate::env::LOCALAI_URL;
use crate::{Error, AI_CONTEXT, HTTP_CLIENT, KV_DATABASE};

use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ModelMessageData {
    pub role: String,
    pub content: String,
}

impl Default for ModelMessageData {
    fn default() -> Self {
        ModelMessageData {
            role: "system".to_string(),
            content: "Given the following conversation, relevant context, and a follow up question, reply with an answer to the current question the user is asking. Return only your response to the question given the above information following the users instructions as needed.".to_string()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaOptions {
    pub num_ctx: u64,
    pub temperature: f64,
    pub num_predict: u64,
}

impl Default for OllamaOptions {
    fn default() -> Self {
        OllamaOptions {
            num_ctx: 8192,
            temperature: 0.7,
            num_predict: 250,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelMessageContext {
    pub messages: Vec<ModelMessageData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelData {
    pub model: String,
    pub messages: VecDeque<ModelMessageData>,
    pub options: OllamaOptions,
}

impl ModelData {
    fn new() -> Self {
        ModelData::default()
    }
}

impl Default for ModelData {
    fn default() -> Self {
        ModelData {
            model: "deepseek-r1:14b".to_string(),
            messages: VecDeque::new(),
            options: OllamaOptions::default(),
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

pub async fn wipe_context(message: &serenity::Message) -> Result<(), Error> {
    let db = KV_DATABASE.get().unwrap();
    let tx = db.begin_write()?;
    {
        let mut tx_table = tx.open_table(AI_CONTEXT)?;
        let channel_id: &str = &message.channel_id.to_string();
        tx_table.remove(channel_id)?;
    }
    tx.commit()?;
    log::info!("Context Cleared for {:?}", message.channel_id);

    Ok(())
}

pub async fn get_gpt_response(
    message: &serenity::Message,
    ctx: &serenity::Context,
) -> Result<String, Error> {
    let db = KV_DATABASE.get().unwrap();
    let rx = db.begin_read()?;
    let rx_table = rx.open_table(AI_CONTEXT)?;

    let mut map = ModelData::new();

    let channel_id: &str = &message.channel_id.to_string();

    if let Some(stored_value) = rx_table.get(channel_id)? {
        let mut stored_messages: VecDeque<ModelMessageData> =
            serde_json::from_str(stored_value.value())?;
        map.messages.append(&mut stored_messages);
    }

    let new_msg = ModelMessageData {
        role: "user".to_string(),
        content: format!(
            "{}",
            message
                .content_safe(&ctx.cache)
                .replace("@Rin#7236", "")
                .trim_start()
        ),
    };
    map.messages.push_back(new_msg);

    log::info!("GPT Sent {:#?}", map.messages);
    let resp = HTTP_CLIENT
        .get()
        .unwrap()
        .post(format!("{}/v1/chat/completions", &*LOCALAI_URL))
        .json(&map)
        .send()
        .await?;

    let json_string = resp.text().await?;

    // Deserialize the JSON string into a Value
    let results: Result<ModelResponse, serde_json::Error> =
        serde_json::from_str(json_string.as_str());

    let model_response = results.unwrap();

    log::info!("GPT Response: {:#?}", model_response);
    // Deepseek has a think section, which should be removed
    let bot_msg = ModelMessageData {
        role: "assistant".to_string(),
        content: model_response.choices[0].message.content.clone(),
    };
    map.messages.push_back(bot_msg);
    log::info!("Length of Context: {}", map.messages.len());
    while map.messages.len() >= 100 {
        map.messages.pop_front();
    }
    let mut content = model_response.choices[0]
        .message
        .content
        .trim_end_matches("<｜end▁of▁sentence｜>")
        .split("</think>")
        .last()
        .unwrap()
        .to_string();
    content.truncate(2000);

    let tx = db.begin_write()?;
    {
        let data = serde_json::to_string(&map.messages)?;
        let mut table = tx.open_table(AI_CONTEXT)?;
        let _ = table.insert(channel_id, data.as_str());
    }
    tx.commit()?;

    Ok(content)
}
