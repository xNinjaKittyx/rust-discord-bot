use actix_files::{Files, NamedFile};
use actix_web::{delete, get, post, put, web, App, HttpResponse, HttpServer, Responder, Result as ActixResult};

use miette::Result;
use redb::ReadableDatabase;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
struct TagRequest {
    key: String,
    value: String,
}

#[get("/api/stats")]
async fn stats() -> impl Responder {
    let db = match crate::KV_DATABASE.get() {
        Some(db) => db,
        None => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))
    };

    // Count tags
    let tag_count = {
        let read_txn = db.begin_read().unwrap();
        match read_txn.open_table(crate::TABLE) {
            Ok(table) => table.range::<&str>(..).unwrap().count() as u64,
            Err(_) => 0
        }
    };

    // Count emojis
    let emoji_count = crate::EMOJIS_LIST.get()
        .map(|emojis_list| emojis_list.len())
        .unwrap_or(0);

    // Count servers (guilds)
    let server_count = crate::EMOJIS_LIST.get()
        .map(|emojis_list| {
            emojis_list.iter()
                .filter_map(|e| e.get("guild_id").and_then(|v| v.as_str()))
                .collect::<std::collections::HashSet<_>>()
                .len()
        })
        .unwrap_or(0);

    // Count commands in last 24 hours
    let commands_24h = {
        let read_txn = db.begin_read().unwrap();
        match read_txn.open_table(crate::HISTORY) {
            Ok(table) => {
                let now = chrono::Utc::now();
                let cutoff = now - chrono::Duration::hours(24);

                table.range::<&str>(..)
                    .unwrap()
                    .filter_map(|item| {
                        let (_key, value) = item.ok()?;
                        let value_str = value.value();
                        let entry: serde_json::Value = serde_json::from_str(value_str).ok()?;
                        let timestamp_str = entry.get("timestamp")?.as_str()?;
                        let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp_str).ok()?;
                        if timestamp.with_timezone(&chrono::Utc) >= cutoff {
                            Some(())
                        } else {
                            None
                        }
                    })
                    .count()
            }
            Err(_) => 0
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "servers": server_count,
        "commands_24h": commands_24h,
        "tags": tag_count,
        "emojis": emoji_count
    }))
}

#[get("/api/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok"
    }))
}

#[get("/api/logs")]
async fn logs() -> impl Responder {
    match std::fs::File::open("bot.log") {
        Ok(file) => {
            let reader = BufReader::new(file);
            let lines: Vec<String> = reader
                .lines()
                .filter_map(|line| line.ok())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .take(1000)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();

            HttpResponse::Ok().json(serde_json::json!({
                "logs": lines,
                "count": lines.len()
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to read log file: {}", e)
        }))
    }
}

#[get("/api/history")]
async fn history() -> impl Responder {
    let db = match crate::KV_DATABASE.get() {
        Some(db) => db,
        None => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))
    };

    let read_txn = match db.begin_read() {
        Ok(txn) => txn,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to read database: {}", e)
        }))
    };

    let history_entries: Vec<serde_json::Value> = {
        let table = match read_txn.open_table(crate::HISTORY) {
            Ok(t) => t,
            Err(_) => return HttpResponse::Ok().json(serde_json::json!({
                "history": [],
                "count": 0
            }))
        };

        table.range::<&str>(..)
            .unwrap()
            .filter_map(|item| {
                let (_key, value) = item.ok()?;
                let value_str = value.value();
                serde_json::from_str(value_str).ok()
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .take(100)
            .collect()
    };

    HttpResponse::Ok().json(serde_json::json!({
        "history": history_entries,
        "count": history_entries.len()
    }))
}

#[get("/api/commands")]
async fn commands() -> impl Responder {
    let commands = crate::COMMANDS_LIST.get()
        .map(|cmds| cmds.clone())
        .unwrap_or_else(Vec::new);

    HttpResponse::Ok().json(serde_json::json!({
        "commands": commands,
        "count": commands.len()
    }))
}

#[get("/api/emojis")]
async fn emojis() -> impl Responder {
    let emojis = crate::EMOJIS_LIST.get()
        .map(|emjs| emjs.clone())
        .unwrap_or_else(Vec::new);

    HttpResponse::Ok().json(serde_json::json!({
        "emojis": emojis,
        "count": emojis.len()
    }))
}

#[get("/api/tags")]
async fn get_tags() -> impl Responder {
    let db = match crate::KV_DATABASE.get() {
        Some(db) => db,
        None => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))
    };

    match db.begin_read() {
        Ok(tx) => {
            match tx.open_table(crate::TABLE) {
                Ok(table) => {
                    let mut tags = Vec::new();
                    if let Ok(iter) = table.range::<&str>(..) {
                        for item in iter {
                            if let Ok((key, value)) = item {
                                tags.push(serde_json::json!({
                                    "key": key.value().to_string(),
                                    "value": value.value().to_string()
                                }));
                            }
                        }
                    }
                    HttpResponse::Ok().json(serde_json::json!({
                        "tags": tags,
                        "count": tags.len()
                    }))
                }
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to open table: {}", e)
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to begin read transaction: {}", e)
        }))
    }
}

#[post("/api/tags")]
async fn create_tag(tag: web::Json<TagRequest>) -> impl Responder {
    let db = match crate::KV_DATABASE.get() {
        Some(db) => db,
        None => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))
    };

    match db.begin_write() {
        Ok(tx) => {
            {
                match tx.open_table(crate::TABLE) {
                    Ok(mut table) => {
                        if let Err(e) = table.insert(tag.key.as_str(), tag.value.as_str()) {
                            return HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": format!("Failed to insert tag: {}", e)
                            }))
                        }
                    }
                    Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to open table: {}", e)
                    }))
                }
            }

            match tx.commit() {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "key": tag.key,
                    "value": tag.value
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to commit transaction: {}", e)
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to begin write transaction: {}", e)
        }))
    }
}

#[put("/api/tags/{key}")]
async fn update_tag(key: web::Path<String>, tag: web::Json<TagRequest>) -> impl Responder {
    let db = match crate::KV_DATABASE.get() {
        Some(db) => db,
        None => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))
    };

    match db.begin_write() {
        Ok(tx) => {
            {
                match tx.open_table(crate::TABLE) {
                    Ok(mut table) => {
                        // Remove old key if it's different from new key
                        if key.as_str() != tag.key.as_str() {
                            let _ = table.remove(key.as_str());
                        }
                        if let Err(e) = table.insert(tag.key.as_str(), tag.value.as_str()) {
                            return HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": format!("Failed to update tag: {}", e)
                            }))
                        }
                    }
                    Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to open table: {}", e)
                    }))
                }
            }

            match tx.commit() {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "key": tag.key,
                    "value": tag.value
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to commit transaction: {}", e)
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to begin write transaction: {}", e)
        }))
    }
}

#[delete("/api/tags/{key}")]
async fn delete_tag(key: web::Path<String>) -> impl Responder {
    let db = match crate::KV_DATABASE.get() {
        Some(db) => db,
        None => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))
    };

    match db.begin_write() {
        Ok(tx) => {
            {
                match tx.open_table(crate::TABLE) {
                    Ok(mut table) => {
                        if let Err(e) = table.remove(key.as_str()) {
                            return HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": format!("Failed to delete tag: {}", e)
                            }))
                        }
                    }
                    Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to open table: {}", e)
                    }))
                }
            }

            match tx.commit() {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "key": key.as_str()
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to commit transaction: {}", e)
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to begin write transaction: {}", e)
        }))
    }
}

async fn spa_fallback() -> ActixResult<NamedFile> {
    Ok(NamedFile::open(PathBuf::from("./static/index.html"))?)
}

pub async fn start_web_server(subsys: &mut tokio_graceful_shutdown::SubsystemHandle) -> Result<()> {
    log::info!("Starting web server...");

    let server = HttpServer::new(|| {
        App::new()
            .service(stats)
            .service(health)
            .service(logs)
            .service(commands)
            .service(emojis)
            .service(history)
            .service(get_tags)
            .service(create_tag)
            .service(update_tag)
            .service(delete_tag)
            .service(
                Files::new("/", "./static")
                    .index_file("index.html")
                    .use_last_modified(true)
                    .default_handler(web::to(spa_fallback))
            )
    })
    .bind(("0.0.0.0", 8080))
    .map_err(|e| miette::miette!("Failed to bind web server: {}", e))?;

    log::info!("Web server listening on http://0.0.0.0:8080");

    let server_handle = server.run();
    let server_task = tokio::spawn(server_handle);

    subsys.on_shutdown_requested().await;
    log::info!("Web server shutting down...");

    server_task.abort();

    Ok(())
}
