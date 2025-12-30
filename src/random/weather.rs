use crate::env::OPENWEATHERMAP_API_KEY;
use crate::{Context, Error, HTTP_CLIENT};

use poise::serenity_prelude as serenity;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct GeoLocation {
    lat: f64,
    lon: f64,
    name: String,
    #[serde(default)]
    country: String,
}

#[derive(Deserialize, Debug)]
struct WeatherCondition {
    main: String,
    description: String,
}

#[derive(Deserialize, Debug)]
struct WeatherData {
    name: String,
    main: Main,
    weather: Vec<WeatherCondition>,
}

#[derive(Deserialize, Debug)]
struct Main {
    temp: f64,
    feels_like: f64,
    temp_min: f64,
    temp_max: f64,
    pressure: f64,
    humidity: f64,
}

async fn get_location(query: &str) -> Result<GeoLocation, Error> {
    let client = HTTP_CLIENT.get().unwrap();
    let api_key = &*OPENWEATHERMAP_API_KEY;

    // Check if the query is a number (zip code)
    let is_zip = query.chars().all(|c| c.is_numeric());

    let url = if is_zip {
        format!(
            "http://api.openweathermap.org/geo/1.0/zip?zip={}&appid={}",
            query, api_key
        )
    } else {
        format!(
            "http://api.openweathermap.org/geo/1.0/direct?q={}&limit=1&appid={}",
            query, api_key
        )
    };

    let response = client.get(&url).send().await?;

    if is_zip {
        let location: GeoLocation = response.json().await?;
        Ok(location)
    } else {
        let locations: Vec<GeoLocation> = response.json().await?;
        locations.into_iter().next().ok_or("Location not found".into())
    }
}

async fn get_weather(lat: f64, lon: f64) -> Result<WeatherData, Error> {
    let client = HTTP_CLIENT.get().unwrap();
    let api_key = &*OPENWEATHERMAP_API_KEY;

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&units=imperial&appid={}",
        lat, lon, api_key
    );

    let response = client.get(&url).send().await?;
    let weather: WeatherData = response.json().await?;

    Ok(weather)
}

#[poise::command(prefix_command, slash_command, category = "Random")]
pub async fn weather(
    ctx: Context<'_>,
    #[description = "City name or ZIP code"] location: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    // Get coordinates from location
    let geo = get_location(&location).await?;

    // Get weather data
    let weather_data = get_weather(geo.lat, geo.lon).await?;

    let unknown = "Unknown".to_string();
    let condition = weather_data.weather.first()
        .map(|w| &w.description)
        .unwrap_or(&unknown);

    let embed = serenity::CreateEmbed::new()
        .title(format!("Weather for {}", weather_data.name))
        .description(condition)
        .field("Current", format!("{:.1}°F", weather_data.main.temp), true)
        .field("Feels Like", format!("{:.1}°F", weather_data.main.feels_like), true)
        .field("Condition", condition, true)
        .field("High", format!("{:.1}°F", weather_data.main.temp_max), true)
        .field("Low", format!("{:.1}°F", weather_data.main.temp_min), true)
        .field("\u{200b}", "\u{200b}", true)
        .field("Humidity", format!("{}%", weather_data.main.humidity), true)
        .field("Pressure", format!("{} hPa", weather_data.main.pressure), true)
        .color(0x5dadec)
        .timestamp(serenity::model::Timestamp::now());

    let reply = poise::CreateReply::default().embed(embed);
    ctx.send(reply).await?;

    Ok(())
}
