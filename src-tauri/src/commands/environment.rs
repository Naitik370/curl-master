use crate::db;

// ---------------------------------------------------------------------------
// Environment & variable commands
// ---------------------------------------------------------------------------

/// Create a new environment in a workspace.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn create_environment(name: String, workspaceId: String) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    db::create_environment(&id, &workspaceId, &name)
        .await
        .map_err(|e| e.to_string())?;
    Ok(id)
}

/// Get all environments in a workspace.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn get_environments(workspaceId: String) -> Result<serde_json::Value, String> {
    let environments = db::get_environments(&workspaceId)
        .await
        .map_err(|e| e.to_string())?;

    let list: Vec<serde_json::Value> = environments
        .into_iter()
        .map(|(id, name, is_active)| {
            serde_json::json!({
                "id": id,
                "name": name,
                "is_active": is_active
            })
        })
        .collect();

    Ok(serde_json::json!(list))
}

/// Set (or clear) the active environment for a workspace.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn set_active_env(id: Option<String>, workspaceId: String) -> Result<(), String> {
    db::set_active_environment(id.as_deref(), &workspaceId)
        .await
        .map_err(|e| e.to_string())
}

/// Get variables for a specific environment.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn get_variables(environmentId: String) -> Result<serde_json::Value, String> {
    let variables = db::get_variables(&environmentId)
        .await
        .map_err(|e| e.to_string())?;

    let list: Vec<serde_json::Value> = variables
        .into_iter()
        .map(|(id, key, value, is_secret)| {
            serde_json::json!({
                "id": id,
                "key": key,
                "value": value,
                "is_secret": is_secret
            })
        })
        .collect();

    Ok(serde_json::json!(list))
}

/// Get the active environment's variables as a key-value map.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn get_active_variables(workspaceId: String) -> Result<serde_json::Value, String> {
    let variables = db::get_active_variables(&workspaceId)
        .await
        .map_err(|e| e.to_string())?;

    let mut map = serde_json::Map::new();
    for (key, value) in variables {
        map.insert(key, serde_json::Value::String(value));
    }

    Ok(serde_json::Value::Object(map))
}

/// Upsert multiple variables for an environment.
///
/// Each variable in `vars` must have `key` and `value` fields. `id` is
/// auto-generated if missing, and `is_secret` defaults to `false`.
#[allow(non_snake_case)]
#[tauri::command]
pub async fn save_variables(
    environmentId: String,
    vars: Vec<serde_json::Value>,
) -> Result<(), String> {
    for v in vars {
        let id = v["id"]
            .as_str()
            .unwrap_or(&uuid::Uuid::new_v4().to_string())
            .to_string();
        let key = v["key"].as_str().unwrap_or("");
        let value = v["value"].as_str().unwrap_or("");
        let is_secret = v["is_secret"].as_bool().unwrap_or(false);

        db::upsert_variable(&id, &environmentId, key, value, is_secret)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
