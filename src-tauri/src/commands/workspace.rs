use crate::db;

// ---------------------------------------------------------------------------
// Workspace commands
// ---------------------------------------------------------------------------

/// Ensure a workspace exists (creates one using the ID as name if missing).
#[allow(non_snake_case)]
#[tauri::command]
pub async fn ensure_workspace(workspaceId: String) -> Result<String, String> {
    db::ensure_workspace(&workspaceId)
        .await
        .map_err(|e| e.to_string())
}

/// Get all workspaces ordered by creation date.
#[tauri::command]
pub async fn get_workspaces() -> Result<serde_json::Value, String> {
    let workspaces = db::get_workspaces()
        .await
        .map_err(|e| e.to_string())?;

    let list: Vec<serde_json::Value> = workspaces
        .into_iter()
        .map(|(id, name)| {
            serde_json::json!({
                "id": id,
                "name": name
            })
        })
        .collect();

    Ok(serde_json::json!(list))
}

/// Create a new workspace with a generated UUID.
///
/// Returns an error if a workspace with the same name already exists.
#[tauri::command]
pub async fn create_workspace(name: String) -> Result<String, String> {
    let exists = db::workspace_name_exists(&name)
        .await
        .map_err(|e| e.to_string())?;
    if exists {
        return Err(format!("A workspace named \"{}\" already exists.", name));
    }

    let id = uuid::Uuid::new_v4().to_string();
    db::create_workspace(&id, &name)
        .await
        .map_err(|e| e.to_string())?;
    Ok(id)
}

/// Rename an existing workspace.
///
/// Returns an error if another workspace with the same name already exists.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn rename_workspace(workspaceId: String, name: String) -> Result<(), String> {
    let exists = db::workspace_name_exists(&name)
        .await
        .map_err(|e| e.to_string())?;
    if exists {
        return Err(format!("A workspace named \"{}\" already exists.", name));
    }

    db::update_workspace_name(&workspaceId, &name)
        .await
        .map_err(|e| e.to_string())
}

/// Delete a workspace and all of its child data.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn delete_workspace(workspaceId: String) -> Result<(), String> {
    db::delete_workspace(&workspaceId)
        .await
        .map_err(|e| e.to_string())
}

/// Clear all app data and reset settings to defaults.
#[tauri::command]
pub async fn clear_all_data() -> Result<(), String> {
    db::clear_all_data()
        .await
        .map_err(|e| e.to_string())
}
