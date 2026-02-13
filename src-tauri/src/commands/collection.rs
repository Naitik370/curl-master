use crate::{db, models::ImportCollection};

// ---------------------------------------------------------------------------
// Collection commands
// ---------------------------------------------------------------------------

/// Create a new collection within a workspace.
///
/// Returns an error if a collection with the same name already exists in this workspace.
/// Automatically assigns the next `sort_order` based on existing collection count.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn create_collection(name: String, workspaceId: String) -> Result<String, String> {
    println!("Creating collection: {} in workspace: {}", name, workspaceId);

    // Duplicate name check (case-insensitive)
    let exists = db::collection_name_exists_in_workspace(&workspaceId, &name)
        .await
        .map_err(|e| e.to_string())?;
    if exists {
        return Err(format!("A collection named \"{}\" already exists in this workspace.", name));
    }

    let id = uuid::Uuid::new_v4().to_string();

    let collections = db::get_collections(&workspaceId)
        .await
        .map_err(|e| e.to_string())?;
    let sort_order = collections.len() as i64;

    db::create_collection(&id, &workspaceId, &name, sort_order)
        .await
        .map_err(|e| e.to_string())?;

    Ok(id)
}

/// Create a new folder inside a collection.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn create_folder(name: String, collectionId: String) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();

    let folders = db::get_folders(&collectionId)
        .await
        .map_err(|e| e.to_string())?;
    let sort_order = folders.len() as i64;

    db::create_folder(&id, &collectionId, &name, sort_order)
        .await
        .map_err(|e| e.to_string())?;

    Ok(id)
}

/// Save a new request under a collection (optionally inside a folder).
///
/// Returns an error if a request with the same name AND method already exists
/// in this collection. Same name with a different method is allowed.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn save_request(
    name: String,
    method: String,
    url: String,
    collectionId: String,
    folderId: Option<String>,
    headers: String,
    params: String,
    body: String,
    auth: Option<String>,
) -> Result<String, String> {
    // Duplicate name+method check within the same collection
    let exists = db::request_name_method_exists_in_collection(&collectionId, &name, &method)
        .await
        .map_err(|e| e.to_string())?;
    if exists {
        return Err(format!(
            "A {} request named \"{}\" already exists in this collection.",
            method, name
        ));
    }

    let id = uuid::Uuid::new_v4().to_string();

    let requests = db::get_requests(&collectionId)
        .await
        .map_err(|e| e.to_string())?;
    let sort_order = requests.len() as i64;

    db::save_request(
        &id,
        &collectionId,
        folderId.as_deref(),
        &name,
        &method,
        &url,
        &headers,
        &params,
        &body,
        auth.as_deref(),
        sort_order,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(id)
}

/// Update an existing request's fields.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn update_request(
    id: String,
    name: String,
    method: String,
    url: String,
    headers: String,
    params: String,
    body: String,
    auth: Option<String>,
) -> Result<(), String> {
    db::update_request(&id, &name, &method, &url, &headers, &params, &body, auth.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Delete a request by ID.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn delete_request(requestId: String) -> Result<(), String> {
    db::delete_request(&requestId)
        .await
        .map_err(|e| e.to_string())
}

/// Delete a collection and all of its folders and requests.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn delete_collection(collectionId: String) -> Result<(), String> {
    db::delete_collection(&collectionId)
        .await
        .map_err(|e| e.to_string())
}

/// Delete a folder and its requests.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn delete_folder(folderId: String) -> Result<(), String> {
    db::delete_folder(&folderId)
        .await
        .map_err(|e| e.to_string())
}

/// Get all collections with their nested folders and requests for a workspace.
///
/// The heavy JSON mapping is delegated to `request_row_to_json` for DRY compliance.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn get_collections_with_folders(
    workspaceId: String,
) -> Result<serde_json::Value, String> {
    let collections = db::get_collections(&workspaceId)
        .await
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();

    for (id, name, _) in collections {
        let folders = db::get_folders(&id)
            .await
            .map_err(|e| e.to_string())?;

        let all_requests = db::get_requests(&id)
            .await
            .map_err(|e| e.to_string())?;

        // Build folder list with their requests
        let folder_list: Vec<serde_json::Value> = folders
            .iter()
            .map(|(f_id, f_name, _)| {
                let folder_requests: Vec<serde_json::Value> = all_requests
                    .iter()
                    .filter(|r| &r.4 == f_id) // r.4 = folder_id
                    .map(request_row_to_json)
                    .collect();

                serde_json::json!({
                    "id": f_id,
                    "name": f_name,
                    "requests": folder_requests
                })
            })
            .collect();

        // Root-level requests (no folder_id)
        let root_requests: Vec<serde_json::Value> = all_requests
            .iter()
            .filter(|r| r.4.is_empty())
            .map(request_row_to_json)
            .collect();

        result.push(serde_json::json!({
            "id": id,
            "name": name,
            "folders": folder_list,
            "requests": root_requests
        }));
    }

    Ok(serde_json::json!(result))
}

/// Batch import a collection with folders and requests.
#[tauri::command]
pub async fn import_collection(collection: ImportCollection) -> Result<String, String> {
    db::import_collection(collection)
        .await
        .map_err(|e| e.to_string())
}

/// Sync-import a collection: replaces any existing collection with the same name.
///
/// Used by the GitHub pull flow to update workspace contents without creating
/// duplicate collections.
#[tauri::command]
pub async fn sync_import_collection(collection: ImportCollection) -> Result<String, String> {
    db::sync_import_collection(collection)
        .await
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Convert a raw request DB row (9-element tuple) into a JSON value.
///
/// This eliminates the copy-pasted mapping that previously appeared in both
/// `get_collections_with_folders` and `get_collection_with_contents` in db.rs.
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
