use poise::serenity_prelude as serenity;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Deserialize, Debug)]
pub struct ReactionConfig {
    pub enabled: bool,
    pub animated: Option<bool>,
    pub emoji_id: Option<u64>,
    pub emoji_name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ReplyConfig {
    pub enabled: bool,
    pub text: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    pub contains: Vec<String>,
    pub case_sensitive: bool,
    pub exact_match: bool,
    pub author: Option<Vec<serenity::UserId>>,
    pub reaction: Option<ReactionConfig>,
    pub reply: Option<ReplyConfig>,
}

#[derive(Deserialize, Debug)]
pub struct AiConfig {
    pub system_prompt: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub response: HashMap<String, Response>,
    pub ai: Option<AiConfig>,
}

pub fn load_config() -> Result<Config, Error> {
    let config_content = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_content)?;
    Ok(config)
}
