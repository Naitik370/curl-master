use crate::{db, http};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Settings commands
// ---------------------------------------------------------------------------

/// Get all settings as a key-value map.
#[tauri::command]
pub async fn get_settings() -> Result<HashMap<String, String>, String> {
    let settings = db::get_all_settings()
        .await
        .map_err(|e| e.to_string())?;

    let map: HashMap<String, String> = settings
        .into_iter()
        .map(|s| (s.key, s.value))
        .collect();

    Ok(map)
}

/// Update a single setting by key.
///
/// When `max_redirects` changes the HTTP executor is rebuilt automatically.
#[tauri::command]
pub async fn update_setting(key: String, value: String) -> Result<(), String> {
    db::update_setting(&key, &value)
        .await
        .map_err(|e| e.to_string())?;

    // Side-effect: rebuild HTTP executor when redirect limit changes
    if key == "max_redirects" {
        if let Ok(max_redirects) = value.parse::<usize>() {
            http::rebuild_executor(max_redirects).await;
        }
    }

    Ok(())
}
