use actix_web::{HttpResponse, Responder, get};

#[get("/api/aydy")]
pub async fn get_aydy() -> impl Responder {
    match crate::db::read_table(crate::AYDY, |key, value| {
        // Parse the JSON value to extract AYDY state
        serde_json::from_str::<serde_json::Value>(value)
            .ok()
            .and_then(|state| {
                let mut state_obj = state.as_object().cloned()?;
                state_obj.insert("channel_key".to_string(), serde_json::json!(key));
                Some(serde_json::Value::Object(state_obj))
            })
    }) {
        Ok(aydy_states) => HttpResponse::Ok().json(serde_json::json!({
            "aydy": aydy_states,
            "count": aydy_states.len()
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}
