use actix_web::{HttpResponse, Responder, get};
use redb::ReadableDatabase;
use std::io::{BufRead, BufReader};

#[get("/api/stats")]
pub async fn stats() -> impl Responder {
    // Count tags
    let tag_count = crate::db::count_entries(crate::TABLE).unwrap_or(0);

    // Count emojis
    let emoji_count = crate::EMOJIS_LIST
        .get()
        .map(|emojis_list| emojis_list.len())
        .unwrap_or(0);

    // Count servers (guilds)
    let server_count = crate::EMOJIS_LIST
        .get()
        .map(|emojis_list| {
            emojis_list
                .iter()
                .filter_map(|e| e.get("guild_id").and_then(|v| v.as_str()))
                .collect::<std::collections::HashSet<_>>()
                .len()
        })
        .unwrap_or(0);

    // Count commands in last 24 hours
    let commands_24h = {
        let db = match crate::KV_DATABASE.get() {
            Some(db) => db,
            None => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Database not initialized"
                }));
            }
        };

        let read_txn = db.begin_read().unwrap();
        match read_txn.open_table(crate::HISTORY) {
            Ok(table) => {
                let now = chrono::Utc::now();
                let cutoff = now - chrono::Duration::hours(24);

                table
                    .range::<&str>(..)
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
            Err(_) => 0,
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
pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok"
    }))
}

#[get("/api/logs")]
pub async fn logs() -> impl Responder {
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
        })),
    }
}

#[get("/api/history")]
pub async fn history() -> impl Responder {
    let db = match crate::KV_DATABASE.get() {
        Some(db) => db,
        None => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database not initialized"
            }));
        }
    };

    let read_txn = match db.begin_read() {
        Ok(txn) => txn,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to read database: {}", e)
            }));
        }
    };

    let history_entries: Vec<serde_json::Value> = {
        let table = match read_txn.open_table(crate::HISTORY) {
            Ok(t) => t,
            Err(_) => {
                return HttpResponse::Ok().json(serde_json::json!({
                    "history": [],
                    "count": 0
                }));
            }
        };

        table
            .range::<&str>(..)
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
pub async fn commands() -> impl Responder {
    let commands = crate::COMMANDS_LIST.get().cloned().unwrap_or_default();

    HttpResponse::Ok().json(serde_json::json!({
        "commands": commands,
        "count": commands.len()
    }))
}

#[get("/api/emojis")]
pub async fn emojis() -> impl Responder {
    let emojis = crate::EMOJIS_LIST.get().cloned().unwrap_or_default();

    HttpResponse::Ok().json(serde_json::json!({
        "emojis": emojis,
        "count": emojis.len()
    }))
}
