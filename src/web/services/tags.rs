use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TagRequest {
    pub key: String,
    pub value: String,
}

#[get("/api/tags")]
pub async fn get_tags() -> impl Responder {
    match crate::db::read_table(crate::TABLE, |key, value| {
        Some(serde_json::json!({
            "key": key,
            "value": value
        }))
    }) {
        Ok(tags) => HttpResponse::Ok().json(serde_json::json!({
            "tags": tags,
            "count": tags.len()
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

#[post("/api/tags")]
pub async fn create_tag(tag: web::Json<TagRequest>) -> impl Responder {
    match crate::db::write_entry(crate::TABLE, &tag.key, &tag.value) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "key": tag.key,
            "value": tag.value
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

#[put("/api/tags/{key}")]
pub async fn update_tag(key: web::Path<String>, tag: web::Json<TagRequest>) -> impl Responder {
    match crate::db::update_entry(crate::TABLE, &key, &tag.key, &tag.value) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "key": tag.key,
            "value": tag.value
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

#[delete("/api/tags/{key}")]
pub async fn delete_tag(key: web::Path<String>) -> impl Responder {
    match crate::db::delete_entry(crate::TABLE, &key) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "key": key.as_str()
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}
