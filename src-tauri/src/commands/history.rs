use crate::db;

// ---------------------------------------------------------------------------
// History commands
// ---------------------------------------------------------------------------

/// Get request history for a workspace with pagination.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn get_history(
    workspaceId: String,
    limit: i64,
    offset: i64,
) -> Result<serde_json::Value, String> {
    let history = db::get_history(&workspaceId, limit, offset)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(history).map_err(|e| e.to_string())
}

/// Clear all history entries for a workspace.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn clear_history(workspaceId: String) -> Result<(), String> {
    db::clear_history(&workspaceId)
        .await
        .map_err(|e| e.to_string())
}

/// Delete a single history entry by ID.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn delete_history_entry(historyId: String) -> Result<(), String> {
    db::delete_history_entry(&historyId)
        .await
        .map_err(|e| e.to_string())
}
