use redb::{Database, ReadableDatabase, TableDefinition};
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum DbError {
    NotInitialized,
    ReadTransaction(String),
    WriteTransaction(String),
    TableOpen(String),
    Query(String),
    Insert(String),
    Delete(String),
    #[allow(dead_code)]
    Serialization(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::NotInitialized => write!(f, "Database not initialized"),
            DbError::ReadTransaction(e) => write!(f, "Failed to begin read transaction: {}", e),
            DbError::WriteTransaction(e) => write!(f, "Failed to begin write transaction: {}", e),
            DbError::TableOpen(e) => write!(f, "Failed to open table: {}", e),
            DbError::Query(e) => write!(f, "Failed to query: {}", e),
            DbError::Insert(e) => write!(f, "Failed to insert: {}", e),
            DbError::Delete(e) => write!(f, "Failed to delete: {}", e),
            DbError::Serialization(e) => write!(f, "Failed to serialize/deserialize: {}", e),
        }
    }
}

impl StdError for DbError {}

/// Get a reference to the global database
pub fn get_db() -> Result<&'static Database, DbError> {
    crate::KV_DATABASE.get().ok_or(DbError::NotInitialized)
}

/// Read all entries from a table and map them to a result type
pub fn read_table<F, T>(
    table_def: TableDefinition<&str, &str>,
    mapper: F,
) -> Result<Vec<T>, DbError>
where
    F: Fn(&str, &str) -> Option<T>,
{
    let db = get_db()?;
    let tx = db
        .begin_read()
        .map_err(|e| DbError::ReadTransaction(e.to_string()))?;
    let table = tx
        .open_table(table_def)
        .map_err(|e| DbError::TableOpen(e.to_string()))?;

    let results = table
        .range::<&str>(..)
        .map_err(|e| DbError::Query(e.to_string()))?
        .filter_map(|item| {
            let (key, value) = item.ok()?;
            mapper(key.value(), value.value())
        })
        .collect();

    Ok(results)
}

/// Read a single entry from a table
#[allow(dead_code)]
pub fn read_entry(
    table_def: TableDefinition<&str, &str>,
    key: &str,
) -> Result<Option<String>, DbError> {
    let db = get_db()?;
    let tx = db
        .begin_read()
        .map_err(|e| DbError::ReadTransaction(e.to_string()))?;
    let table = tx
        .open_table(table_def)
        .map_err(|e| DbError::TableOpen(e.to_string()))?;

    let value = table
        .get(key)
        .map_err(|e| DbError::Query(e.to_string()))?
        .map(|v| v.value().to_string());

    Ok(value)
}

/// Write a single entry to a table
pub fn write_entry(
    table_def: TableDefinition<&str, &str>,
    key: &str,
    value: &str,
) -> Result<(), DbError> {
    let db = get_db()?;
    let tx = db
        .begin_write()
        .map_err(|e| DbError::WriteTransaction(e.to_string()))?;
    {
        let mut table = tx
            .open_table(table_def)
            .map_err(|e| DbError::TableOpen(e.to_string()))?;
        table
            .insert(key, value)
            .map_err(|e| DbError::Insert(e.to_string()))?;
    }
    tx.commit().map_err(|e| DbError::Insert(e.to_string()))?;
    Ok(())
}

/// Delete a single entry from a table
pub fn delete_entry(table_def: TableDefinition<&str, &str>, key: &str) -> Result<(), DbError> {
    let db = get_db()?;
    let tx = db
        .begin_write()
        .map_err(|e| DbError::WriteTransaction(e.to_string()))?;
    {
        let mut table = tx
            .open_table(table_def)
            .map_err(|e| DbError::TableOpen(e.to_string()))?;
        table
            .remove(key)
            .map_err(|e| DbError::Delete(e.to_string()))?;
    }
    tx.commit().map_err(|e| DbError::Delete(e.to_string()))?;
    Ok(())
}

/// Count entries in a table
pub fn count_entries(table_def: TableDefinition<&str, &str>) -> Result<usize, DbError> {
    let db = get_db()?;
    let tx = db
        .begin_read()
        .map_err(|e| DbError::ReadTransaction(e.to_string()))?;
    let table = tx
        .open_table(table_def)
        .map_err(|e| DbError::TableOpen(e.to_string()))?;

    let count = table
        .range::<&str>(..)
        .map_err(|e| DbError::Query(e.to_string()))?
        .count();

    Ok(count)
}

/// Update an entry (delete old key if different, insert new)
pub fn update_entry(
    table_def: TableDefinition<&str, &str>,
    old_key: &str,
    new_key: &str,
    value: &str,
) -> Result<(), DbError> {
    let db = get_db()?;
    let tx = db
        .begin_write()
        .map_err(|e| DbError::WriteTransaction(e.to_string()))?;
    {
        let mut table = tx
            .open_table(table_def)
            .map_err(|e| DbError::TableOpen(e.to_string()))?;

        if old_key != new_key {
            let _ = table.remove(old_key);
        }

        table
            .insert(new_key, value)
            .map_err(|e| DbError::Insert(e.to_string()))?;
    }
    tx.commit().map_err(|e| DbError::Insert(e.to_string()))?;
    Ok(())
}
