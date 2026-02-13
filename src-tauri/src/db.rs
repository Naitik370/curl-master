use crate::models::{Setting, ImportCollection, ImportFolder};
use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;
use futures::FutureExt;
use uuid::Uuid;

lazy_static::lazy_static! {
    static ref DB_POOL: Arc<RwLock<Option<Pool<Sqlite>>>> = Arc::new(RwLock::new(None));
}

/// Initialize database and run migrations
pub async fn init_db(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");

    // Ensure directory exists
    std::fs::create_dir_all(&app_dir)?;

    let db_path = app_dir.join("curlmaster.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    // Create connection pool
    let pool = SqlitePool::connect(&db_url).await?;

    // Enable foreign keys
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;

    // Run migrations
    run_migrations(&pool).await?;

    // Store pool globally
    let mut db_pool = DB_POOL.write().await;
    *db_pool = Some(pool);

    Ok(())
}

/// Get database pool
pub async fn get_pool() -> Result<Pool<Sqlite>, sqlx::Error> {
    DB_POOL.read().await.clone().ok_or(sqlx::Error::PoolClosed)
}

/// Run database migrations
async fn run_migrations(pool: &Pool<Sqlite>) -> Result<(), Box<dyn std::error::Error>> {
    // Create schema_version table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create settings table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create workspace table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS workspace (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create collection table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS collection (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL REFERENCES workspace(id),
            name TEXT NOT NULL,
            description TEXT,
            sort_order INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create folder table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS folder (
            id TEXT PRIMARY KEY,
            collection_id TEXT NOT NULL REFERENCES collection(id),
            parent_folder_id TEXT REFERENCES folder(id),
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create request table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS request (
            id TEXT PRIMARY KEY,
            folder_id TEXT REFERENCES folder(id),
            collection_id TEXT NOT NULL REFERENCES collection(id),
            name TEXT NOT NULL,
            method TEXT NOT NULL,
            url TEXT NOT NULL,
            headers TEXT NOT NULL,
            params TEXT NOT NULL,
            body TEXT NOT NULL,
            auth TEXT,
            sort_order INTEGER NOT NULL,
            schema_version INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create environment table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS environment (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL REFERENCES workspace(id),
            name TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 0
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create variable table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS variable (
            id TEXT PRIMARY KEY,
            environment_id TEXT NOT NULL REFERENCES environment(id),
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            is_secret INTEGER NOT NULL DEFAULT 0
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create history table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS history (
            id TEXT PRIMARY KEY,
            workspace_id TEXT NOT NULL REFERENCES workspace(id),
            request_id TEXT REFERENCES request(id),
            method TEXT NOT NULL,
            url TEXT NOT NULL,
            request_headers TEXT NOT NULL,
            request_params TEXT NOT NULL DEFAULT '[]',
            request_body TEXT,
            response_status INTEGER NOT NULL,
            response_headers TEXT NOT NULL,
            response_body_preview TEXT,
            response_body_full TEXT,
            response_body_type TEXT NOT NULL,
            response_size_bytes INTEGER NOT NULL,
            response_time_ms INTEGER NOT NULL,
            content_type TEXT,
            content_encoding TEXT,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create tab_state table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tab_state (
            id TEXT PRIMARY KEY,
            request_id TEXT REFERENCES request(id),
            draft_data TEXT NOT NULL,
            draft_version INTEGER NOT NULL DEFAULT 1,
            tab_order INTEGER NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 0
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Insert default settings if not exist
    initialize_default_settings(pool).await?;

    // Migration: Add request_params column to history table if it doesn't exist
    // This is for databases created before this column was added
    let _ = sqlx::query(
        r#"
        ALTER TABLE history ADD COLUMN request_params TEXT NOT NULL DEFAULT '[]'
        "#,
    )
    .execute(pool)
    .await; // Ignore error if column already exists

    // Mark migration as complete
    sqlx::query(
        "INSERT OR IGNORE INTO schema_version (version, applied_at) VALUES (1, datetime('now'))",
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Initialize default settings
async fn initialize_default_settings(pool: &Pool<Sqlite>) -> Result<(), Box<dyn std::error::Error>> {
    let defaults = vec![
        ("timeout_ms", "30000"),
        ("max_redirects", "10"),
        ("theme", "dark"),
        ("ignore_tls_default", "false"),
        ("follow_redirects_default", "true"),
        ("history_limit", "100"),
    ];

    for (key, value) in defaults {
        sqlx::query("INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(pool)
            .await?;
    }

    Ok(())
}

/// Get setting by key
pub async fn get_setting(key: &str) -> Result<Option<Setting>, sqlx::Error> {
    let pool = get_pool().await?;
    
    let result = sqlx::query_as::<_, (String, String)>(
        "SELECT key, value FROM settings WHERE key = ?"
    )
    .bind(key)
    .fetch_optional(&pool)
    .await?;

    Ok(result.map(|(key, value)| Setting { key, value }))
}

/// Update setting
pub async fn update_setting(key: &str, value: &str) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;
    
    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
        .bind(key)
        .bind(value)
        .execute(&pool)
        .await?;

    Ok(())
}

/// Get all settings
pub async fn get_all_settings() -> Result<Vec<Setting>, sqlx::Error> {
    let pool = get_pool().await?;
    
    let results = sqlx::query_as::<_, (String, String)>(
        "SELECT key, value FROM settings"
    )
    .fetch_all(&pool)
    .await?;

    Ok(results
        .into_iter()
        .map(|(key, value)| Setting { key, value })
        .collect())
}

// Collection operations
pub async fn create_collection(
    id: &str,
    workspace_id: &str,
    name: &str,
    sort_order: i64,
) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query(
        "INSERT INTO collection (id, workspace_id, name, description, sort_order, created_at)
         VALUES (?, ?, ?, NULL, ?, ?)"
    )
    .bind(id)
    .bind(workspace_id)
    .bind(name)
    .bind(sort_order)
    .bind(now)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Check if a collection with the given name already exists in a workspace.
pub async fn collection_name_exists_in_workspace(workspace_id: &str, name: &str) -> Result<bool, sqlx::Error> {
    let pool = get_pool().await?;
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM collection WHERE workspace_id = ? AND LOWER(name) = LOWER(?)"
    )
    .bind(workspace_id)
    .bind(name)
    .fetch_one(&pool)
    .await?;
    Ok(count.0 > 0)
}

/// Find a collection ID by name within a workspace (for sync/replace).
pub async fn find_collection_id_by_name(workspace_id: &str, name: &str) -> Result<Option<String>, sqlx::Error> {
    let pool = get_pool().await?;
    let result = sqlx::query_scalar::<_, String>(
        "SELECT id FROM collection WHERE workspace_id = ? AND LOWER(name) = LOWER(?) LIMIT 1"
    )
    .bind(workspace_id)
    .bind(name)
    .fetch_optional(&pool)
    .await?;
    Ok(result)
}

pub async fn get_collections(workspace_id: &str) -> Result<Vec<(String, String, String)>, sqlx::Error> {
    let pool = get_pool().await?;

    let results = sqlx::query_as::<_, (String, String, String)>(
        "SELECT id, name, workspace_id FROM collection WHERE workspace_id = ? ORDER BY sort_order"
    )
    .bind(workspace_id)
    .fetch_all(&pool)
    .await?;

    Ok(results)
}

// Folder operations
pub async fn create_folder(
    id: &str,
    collection_id: &str,
    name: &str,
    sort_order: i64,
) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query(
        "INSERT INTO folder (id, collection_id, parent_folder_id, name, sort_order, created_at)
         VALUES (?, ?, NULL, ?, ?, ?)"
    )
    .bind(id)
    .bind(collection_id)
    .bind(name)
    .bind(sort_order)
    .bind(now)
    .execute(&pool)
    .await?;

    Ok(())
}

pub async fn get_folders(collection_id: &str) -> Result<Vec<(String, String, String)>, sqlx::Error> {
    let pool = get_pool().await?;

    let results = sqlx::query_as::<_, (String, String, String)>(
        "SELECT id, name, collection_id FROM folder WHERE collection_id = ? ORDER BY sort_order"
    )
    .bind(collection_id)
    .fetch_all(&pool)
    .await?;

    Ok(results)
}

// Request operations
pub async fn save_request(
    id: &str,
    collection_id: &str,
    folder_id: Option<&str>,
    name: &str,
    method: &str,
    url: &str,
    headers: &str,
    params: &str,
    body: &str,
    auth: Option<&str>,
    sort_order: i64,
) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query(
        "INSERT INTO request (id, folder_id, collection_id, name, method, url, headers, params, body, auth, sort_order, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(id)
    .bind(folder_id)
    .bind(collection_id)
    .bind(name)
    .bind(method)
    .bind(url)
    .bind(headers)
    .bind(params)
    .bind(body)
    .bind(auth)
    .bind(sort_order)
    .bind(now)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Check if a request with the given name+method already exists in a collection.
///
/// Same name but different method is allowed (e.g. GET /users and POST /users).
/// Same name + same method in different collections is also allowed.
pub async fn request_name_method_exists_in_collection(
    collection_id: &str,
    name: &str,
    method: &str,
) -> Result<bool, sqlx::Error> {
    let pool = get_pool().await?;
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM request WHERE collection_id = ? AND LOWER(name) = LOWER(?) AND UPPER(method) = UPPER(?)"
    )
    .bind(collection_id)
    .bind(name)
    .bind(method)
    .fetch_one(&pool)
    .await?;
    Ok(count.0 > 0)
}

pub async fn update_request(
    id: &str,
    name: &str,
    method: &str,
    url: &str,
    headers: &str,
    params: &str,
    body: &str,
    auth: Option<&str>,
) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;
    
    sqlx::query(
        "UPDATE request 
         SET name = ?, method = ?, url = ?, headers = ?, params = ?, body = ?, auth = ? 
         WHERE id = ?"
    )
    .bind(name)
    .bind(method)
    .bind(url)
    .bind(headers)
    .bind(params)
    .bind(body)
    .bind(auth)
    .bind(id)
    .execute(&pool)
    .await?;

    Ok(())
}

pub async fn get_requests(collection_id: &str) -> Result<Vec<(String, String, String, String, String, String, String, String, String)>, sqlx::Error> {
    let pool = get_pool().await?;

    let results = sqlx::query_as::<_, (String, String, String, String, String, String, String, String, String)>(
        "SELECT id, name, method, url, COALESCE(folder_id, '') as folder_id, headers, params, body, COALESCE(auth, '') as auth
         FROM request 
         WHERE collection_id = ? 
         ORDER BY sort_order"
    )
    .bind(collection_id)
    .fetch_all(&pool)
    .await?;

    Ok(results)
}

// Workspace operations
pub async fn ensure_workspace(id: &str) -> Result<String, sqlx::Error> {
    let pool = get_pool().await?;
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query(
        "INSERT OR IGNORE INTO workspace (id, name, created_at, updated_at)
         VALUES (?, ?, ?, ?)"
    )
    .bind(id)
    .bind(id) // Use ID as initial name if it doesn't exist
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await?;

    Ok(id.to_string())
}

pub async fn create_workspace(id: &str, name: &str) -> Result<String, sqlx::Error> {
    let pool = get_pool().await?;
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query(
        "INSERT INTO workspace (id, name, created_at, updated_at)
         VALUES (?, ?, ?, ?)"
    )
    .bind(id)
    .bind(name)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await?;

    Ok(id.to_string())
}

/// Check if a workspace with the given name already exists.
pub async fn workspace_name_exists(name: &str) -> Result<bool, sqlx::Error> {
    let pool = get_pool().await?;
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM workspace WHERE LOWER(name) = LOWER(?)"
    )
    .bind(name)
    .fetch_one(&pool)
    .await?;
    Ok(count.0 > 0)
}

pub async fn get_workspaces() -> Result<Vec<(String, String)>, sqlx::Error> {
    let pool = get_pool().await?;
    
    let results = sqlx::query_as::<_, (String, String)>(
        "SELECT id, name FROM workspace ORDER BY created_at ASC"
    )
    .fetch_all(&pool)
    .await?;
    
    Ok(results)
}
// Environment operations
pub async fn create_environment(
    id: &str,
    workspace_id: &str,
    name: &str,
) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;

    sqlx::query(
        "INSERT INTO environment (id, workspace_id, name, is_active)
         VALUES (?, ?, ?, 0)"
    )
    .bind(id)
    .bind(workspace_id)
    .bind(name)
    .execute(&pool)
    .await?;

    Ok(())
}

pub async fn get_environments(workspace_id: &str) -> Result<Vec<(String, String, bool)>, sqlx::Error> {
    let pool = get_pool().await?;

    let results = sqlx::query_as::<_, (String, String, bool)>(
        "SELECT id, name, is_active FROM environment WHERE workspace_id = ?"
    )
    .bind(workspace_id)
    .fetch_all(&pool)
    .await?;

    Ok(results)
}

pub async fn set_active_environment(id: Option<&str>, workspace_id: &str) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;

    // Deactivate all in workspace
    sqlx::query("UPDATE environment SET is_active = 0 WHERE workspace_id = ?")
        .bind(workspace_id)
        .execute(&pool)
        .await?;

    // Activate the selected one
    if let Some(env_id) = id {
        sqlx::query("UPDATE environment SET is_active = 1 WHERE id = ?")
            .bind(env_id)
            .execute(&pool)
            .await?;
    }

    Ok(())
}

// Variable operations
pub async fn upsert_variable(
    id: &str,
    environment_id: &str,
    key: &str,
    value: &str,
    is_secret: bool,
) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;

    sqlx::query(
        "INSERT INTO variable (id, environment_id, key, value, is_secret)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET key = excluded.key, value = excluded.value, is_secret = excluded.is_secret"
    )
    .bind(id)
    .bind(environment_id)
    .bind(key)
    .bind(value)
    .bind(is_secret as i32)
    .execute(&pool)
    .await?;

    Ok(())
}

pub async fn get_variables(environment_id: &str) -> Result<Vec<(String, String, String, bool)>, sqlx::Error> {
    let pool = get_pool().await?;

    let results = sqlx::query_as::<_, (String, String, String, bool)>(
        "SELECT id, key, value, is_secret FROM variable WHERE environment_id = ?"
    )
    .bind(environment_id)
    .fetch_all(&pool)
    .await?;

    Ok(results)
}

pub async fn get_active_variables(workspace_id: &str) -> Result<Vec<(String, String)>, sqlx::Error> {
    let pool = get_pool().await?;

    // Fetch variables for the active environment in this workspace
    let results = sqlx::query_as::<_, (String, String)>(
        "SELECT v.key, v.value 
         FROM variable v 
         JOIN environment e ON v.environment_id = e.id 
         WHERE e.workspace_id = ? AND e.is_active = 1"
    )
    .bind(workspace_id)
    .fetch_all(&pool)
    .await?;

    Ok(results)
}

/// Clear all user data (workspaces, collections, history, environments, etc.) and reset settings to defaults.
pub async fn clear_all_data() -> Result<(), Box<dyn std::error::Error>> {
    let pool = get_pool().await?;

    // Delete in order respecting foreign keys
    sqlx::query("DELETE FROM variable").execute(&pool).await?;
    sqlx::query("DELETE FROM environment").execute(&pool).await?;
    sqlx::query("DELETE FROM history").execute(&pool).await?;
    sqlx::query("DELETE FROM tab_state").execute(&pool).await?;
    sqlx::query("DELETE FROM request").execute(&pool).await?;
    sqlx::query("DELETE FROM folder").execute(&pool).await?;
    sqlx::query("DELETE FROM collection").execute(&pool).await?;
    sqlx::query("DELETE FROM workspace").execute(&pool).await?;
    sqlx::query("DELETE FROM settings").execute(&pool).await?;

    initialize_default_settings(&pool).await?;
    Ok(())
}

// Delete operations
pub async fn delete_workspace(id: &str) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;
    
    // 1. Delete variables (linked via environments)
    sqlx::query("DELETE FROM variable WHERE environment_id IN (SELECT id FROM environment WHERE workspace_id = ?)")
        .bind(id)
        .execute(&pool)
        .await?;
        
    // 2. Delete environments
    sqlx::query("DELETE FROM environment WHERE workspace_id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
        
    // 3. Delete requests
    // Re-check schema for request... it HAS collection_id.
    sqlx::query("DELETE FROM request WHERE collection_id IN (SELECT id FROM collection WHERE workspace_id = ?)")
        .bind(id)
        .execute(&pool)
        .await?;
        
    // 4. Delete folders
    sqlx::query("DELETE FROM folder WHERE collection_id IN (SELECT id FROM collection WHERE workspace_id = ?)")
        .bind(id)
        .execute(&pool)
        .await?;
        
    // 5. Delete collections
    sqlx::query("DELETE FROM collection WHERE workspace_id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
        
    // 6. Delete workspace itself
    sqlx::query("DELETE FROM workspace WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
        
    Ok(())
}

pub async fn update_workspace_name(id: &str, name: &str) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query("UPDATE workspace SET name = ?, updated_at = ? WHERE id = ?")
        .bind(name)
        .bind(now)
        .bind(id)
        .execute(&pool)
        .await?;

    Ok(())
}

pub async fn delete_collection(id: &str) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;
    
    // Delete requests in this collection
    sqlx::query("DELETE FROM request WHERE collection_id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
        
    // Delete folders in this collection
    sqlx::query("DELETE FROM folder WHERE collection_id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
        
    // Delete collection
    sqlx::query("DELETE FROM collection WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
        
    Ok(())
}

pub async fn delete_folder(id: &str) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;
    
    // Move requests to root of collection or delete them? Usually delete is safer for simple MVP
    sqlx::query("DELETE FROM request WHERE folder_id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
        
    // Delete folder
    sqlx::query("DELETE FROM folder WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
        
    Ok(())
}

pub async fn delete_request(id: &str) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;
    
    sqlx::query("DELETE FROM request WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
        
    Ok(())
}

pub async fn get_collection_with_contents(collection_id: &str) -> Result<serde_json::Value, sqlx::Error> {
    let folders = get_folders(collection_id).await?;
    let all_requests = get_requests(collection_id).await?;

    // Build folder list with their requests
    let folder_list: Vec<serde_json::Value> = folders
        .iter()
        .map(|(f_id, f_name, _)| {
            let folder_requests: Vec<serde_json::Value> = all_requests
                .iter()
                .filter(|r| &r.4 == f_id)
                .map(request_row_to_json)
                .collect();

            serde_json::json!({
                "id": f_id,
                "name": f_name,
                "requests": folder_requests
            })
        })
        .collect();

    // Root requests (no folder_id)
    let root_requests: Vec<serde_json::Value> = all_requests
        .iter()
        .filter(|r| r.4.is_empty())
        .map(request_row_to_json)
        .collect();

    // Fetch collection name
    let pool = get_pool().await?;
    let collection_name = sqlx::query_scalar::<_, String>("SELECT name FROM collection WHERE id = ?")
        .bind(collection_id)
        .fetch_one(&pool)
        .await?;

    Ok(serde_json::json!({
        "id": collection_id,
        "name": collection_name,
        "folders": folder_list,
        "requests": root_requests
    }))
}

/// Convert a raw request DB row (9-element tuple) into a JSON value.
///
/// Centralizes the repeated JSON mapping logic used in collection content queries.
fn request_row_to_json(
    row: &(String, String, String, String, String, String, String, String, String),
) -> serde_json::Value {
    let (r_id, r_name, r_method, r_url, _, r_headers, r_params, r_body, r_auth) = row;

    serde_json::json!({
        "id": r_id,
        "name": r_name,
        "method": r_method,
        "url": r_url,
        "headers": parse_json_or(r_headers, serde_json::json!([])),
        "params": parse_json_or(r_params, serde_json::json!([])),
        "body": parse_json_or(r_body, serde_json::json!({"type": "none", "content": ""})),
        "auth": parse_auth_json(r_auth),
    })
}

/// Parse a JSON string, returning `fallback` on failure.
fn parse_json_or(s: &str, fallback: serde_json::Value) -> serde_json::Value {
    serde_json::from_str::<serde_json::Value>(s).unwrap_or(fallback)
}

/// Parse auth JSON; returns `{"type": "none"}` for empty or invalid strings.
fn parse_auth_json(s: &str) -> serde_json::Value {
    if s.is_empty() {
        serde_json::json!({"type": "none"})
    } else {
        serde_json::from_str::<serde_json::Value>(s)
            .unwrap_or_else(|_| serde_json::json!({"type": "none"}))
    }
}

pub async fn export_workspace(workspace_id: &str) -> Result<serde_json::Value, sqlx::Error> {
    let collections = get_collections(workspace_id).await?;
    let mut collection_data = Vec::new();
    
    for (id, _, _) in collections {
        collection_data.push(get_collection_with_contents(&id).await?);
    }
    
    Ok(serde_json::json!({
        "workspace_id": workspace_id,
        "collections": collection_data
    }))
}

// History operations
pub async fn add_history_entry(
    id: &str,
    workspace_id: &str,
    request_id: Option<&str>,
    method: &str,
    url: &str,
    req_headers: &str,
    req_params: &str,
    req_body: Option<&str>,
    res_status: u16,
    res_headers: &str,
    res_body_preview: Option<&str>,
    res_body_full: Option<&str>,
    res_body_type: &str,
    res_size: i64,
    res_time: i64,
    content_type: Option<&str>,
    content_encoding: Option<&str>,
) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query(
        "INSERT INTO history (
            id, workspace_id, request_id, method, url, 
            request_headers, request_params, request_body, 
            response_status, response_headers, 
            response_body_preview, response_body_full, response_body_type,
            response_size_bytes, response_time_ms, 
            content_type, content_encoding, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(id)
    .bind(workspace_id)
    .bind(request_id)
    .bind(method)
    .bind(url)
    .bind(req_headers)
    .bind(req_params)
    .bind(req_body)
    .bind(res_status)
    .bind(res_headers)
    .bind(res_body_preview)
    .bind(res_body_full)
    .bind(res_body_type)
    .bind(res_size)
    .bind(res_time)
    .bind(content_type)
    .bind(content_encoding)
    .bind(now)
    .execute(&pool)
    .await?;

    Ok(())
}

pub async fn get_history(workspace_id: &str, limit: i64, offset: i64) -> Result<Vec<crate::models::HistoryItem>, sqlx::Error> {
    let pool = get_pool().await?;

    let rows = sqlx::query_as::<_, (
        String, String, Option<String>, String, String,
        String, String, Option<String>,
        i64, String,
        i64, i64, i64
    )>(
        r#"
        SELECT 
            id, workspace_id, request_id, method, url, 
            request_headers, request_params, request_body, 
            response_status, response_headers, 
            response_time_ms, response_size_bytes, created_at
        FROM history 
        WHERE workspace_id = ? 
        ORDER BY created_at DESC 
        LIMIT ? OFFSET ?
        "#
    )
    .bind(workspace_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await?;

    let items = rows.into_iter().map(|(
        id, workspace_id, request_id, method, url,
        request_headers, request_params, request_body,
        response_status, response_headers,
        response_time_ms, response_size_bytes, created_at
    )| {
        crate::models::HistoryItem {
            id,
            workspace_id,
            request_id,
            method,
            url,
            request_headers,
            request_params,
            request_body,
            response_status: response_status as u16,
            response_headers,
            response_time_ms: response_time_ms as u64,
            response_size_bytes: response_size_bytes as usize,
            created_at,
        }
    }).collect();

    Ok(items)
}


pub async fn clear_history(workspace_id: &str) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;
    
    sqlx::query("DELETE FROM history WHERE workspace_id = ?")
        .bind(workspace_id)
        .execute(&pool)
        .await?;
        
    Ok(())
}

pub async fn delete_history_entry(id: &str) -> Result<(), sqlx::Error> {
    let pool = get_pool().await?;
    
    sqlx::query("DELETE FROM history WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
        
    Ok(())
}

pub async fn import_collection(data: ImportCollection) -> Result<String, sqlx::Error> {
    let pool = get_pool().await?;
    let collection_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();

    // Start transaction
    let mut tx = pool.begin().await?;

    // 1. Create Collection
    sqlx::query(
        "INSERT INTO collection (id, workspace_id, name, sort_order, created_at)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&collection_id)
    .bind(&data.workspace_id)
    .bind(&data.name)
    .bind(0)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    // 2. Insert Top-level Requests
    for (i, req) in data.requests.into_iter().enumerate() {
        save_request_internal(
            &mut *tx,
            &Uuid::new_v4().to_string(),
            &collection_id,
            None,
            &req.name,
            &req.method,
            &req.url,
            &req.headers,
            &req.params,
            &req.body,
            Some(&req.auth),
            i as i64,
        ).await?;
    }

    // 3. Recursively Insert Folders
    for (i, folder) in data.folders.into_iter().enumerate() {
        import_folder_recursive_internal(&mut *tx, &collection_id, None, folder, i as i64).await?;
    }

    tx.commit().await?;

    Ok(collection_id)
}

/// Sync-import a collection: if a collection with the same name exists in the
/// workspace, delete it first, then import the new data. This prevents
/// duplicates when pulling from GitHub.
pub async fn sync_import_collection(data: ImportCollection) -> Result<String, sqlx::Error> {
    // Check if a collection with this name already exists in the workspace
    if let Some(existing_id) = find_collection_id_by_name(&data.workspace_id, &data.name).await? {
        // Delete the existing collection and its contents
        delete_collection(&existing_id).await?;
    }

    // Now import fresh
    import_collection(data).await
}

fn import_folder_recursive_internal<'a>(
    conn: &'a mut sqlx::SqliteConnection,
    collection_id: &'a str,
    parent_folder_id: Option<String>,
    folder: ImportFolder,
    sort_order: i64,
) -> futures::future::BoxFuture<'a, Result<(), sqlx::Error>> {
    async move {
        let folder_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis();

        // Insert folder
        sqlx::query(
            "INSERT INTO folder (id, collection_id, parent_folder_id, name, sort_order, created_at)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&folder_id)
        .bind(collection_id)
        .bind(&parent_folder_id)
        .bind(&folder.name)
        .bind(sort_order)
        .bind(now)
        .execute(&mut *conn)
        .await?;

        // Insert requests in folder
        for (i, req) in folder.requests.into_iter().enumerate() {
            save_request_internal(
                &mut *conn,
                &Uuid::new_v4().to_string(),
                collection_id,
                Some(&folder_id),
                &req.name,
                &req.method,
                &req.url,
                &req.headers,
                &req.params,
                &req.body,
                Some(&req.auth),
                i as i64,
            ).await?;
        }

        // Recursively insert subfolders
        for (i, subfolder) in folder.folders.into_iter().enumerate() {
            import_folder_recursive_internal(&mut *conn, collection_id, Some(folder_id.clone()), subfolder, i as i64).await?;
        }

        Ok(())
    }.boxed()
}

pub async fn save_request_internal(
    conn: &mut sqlx::SqliteConnection,
    id: &str,
    collection_id: &str,
    folder_id: Option<&str>,
    name: &str,
    method: &str,
    url: &str,
    headers: &str,
    params: &str,
    body: &str,
    auth: Option<&str>,
    sort_order: i64,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp_millis();

    sqlx::query(
        "INSERT INTO request (id, folder_id, collection_id, name, method, url, headers, params, body, auth, sort_order, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(id)
    .bind(folder_id)
    .bind(collection_id)
    .bind(name)
    .bind(method)
    .bind(url)
    .bind(headers)
    .bind(params)
    .bind(body)
    .bind(auth)
    .bind(sort_order)
    .bind(now)
    .execute(conn)
    .await?;

    Ok(())
}




