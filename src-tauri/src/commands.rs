use crate::{db, http, models::*, sync};
use std::collections::HashMap;

/// Fire-and-forget push to sync server after a mutating command. Stores last error in settings on failure.
fn trigger_push(workspace_id: String) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = sync::push_workspace(&workspace_id).await {
            let _ = db::update_setting("sync_last_error", &e).await;
        }
    });
}

/// Send HTTP request with variable substitution
#[allow(non_snake_case)]
#[tauri::command]
pub async fn send_request(mut config: RequestConfig, workspaceId: String) -> SendResult {
    // 1. Fetch active variables for this workspace
    let variables = match db::get_active_variables(&workspaceId).await {
        Ok(v) => v,
        Err(e) => {
            return SendResult::Failed { 
                request_id: "error".to_string(), 
                error: HttpError::Network { message: format!("Failed to fetch variables: {}", e) }
            };
        }
    };

    // 2. Perform substitution in URL, headers, params, and body
    substitute_variables(&mut config, &variables);

    // 3. Execute request
    let result = http::execute_request(config.clone()).await;

    // 4. Save to history if successful
    if let SendResult::Success { request_id: _, response } = &result {
        // Prepare headers JSON
        let req_headers = serde_json::to_string(&config.headers).unwrap_or_default();
        
        // Prepare params JSON
        let req_params = serde_json::to_string(&config.params).unwrap_or_default();
        
        // Prepare body
        let req_body = match &config.body {
            RequestBody::None => None,
            RequestBody::Raw { content, .. } => Some(content.clone()),
            RequestBody::Json { value } => Some(value.to_string()),
            RequestBody::FormUrlEncoded { fields } => Some(serde_json::to_string(fields).unwrap_or_default()),
            RequestBody::Multipart { fields } => Some(serde_json::to_string(fields).unwrap_or_default()),
        };

        // Prepare response headers JSON
        let res_headers = serde_json::to_string(&response.headers).unwrap_or_default();

        // Prepare response body preview (first 1000 chars)
        let (res_body_full, res_body_preview, res_body_type) = match &response.body {
            ResponseBody::Text { content } => {
                let preview: String = content.chars().take(1000).collect();
                (Some(content.clone()), Some(preview), "text")
            },
            ResponseBody::Binary { preview_hex } => {
                (None, Some(preview_hex.clone()), "binary")
            },
            ResponseBody::Truncated { content, .. } => {
                let preview: String = content.chars().take(1000).collect();
                (Some(content.clone()), Some(preview), "truncated")
            }
        };

        if let Err(e) = db::add_history_entry(
            &uuid::Uuid::new_v4().to_string(),
            &workspaceId,
            None, // Don't link to request table for ad-hoc requests
            &config.method,
            &config.url,
            &req_headers,
            &req_params,
            req_body.as_deref(),
            response.status,
            &res_headers,
            res_body_preview.as_deref(),
            res_body_full.as_deref(),
            res_body_type,
            response.size_bytes as i64,
            response.time_ms as i64,
            response.content_type.as_deref(),
            response.content_encoding.as_deref(),
        ).await {
            eprintln!("Failed to save history: {:?}", e);
        }
    }

    result
}

fn substitute_variables(config: &mut RequestConfig, variables: &[(String, String)]) {
    for (key, value) in variables {
        let pattern = format!("{{{{{}}}}}", key);
        
        // Replace in URL
        config.url = config.url.replace(&pattern, value);
        
        // Replace in Headers
        for header in &mut config.headers {
            header.key = header.key.replace(&pattern, value);
            header.value = header.value.replace(&pattern, value);
        }
        
        // Replace in Params
        for param in &mut config.params {
            param.key = param.key.replace(&pattern, value);
            param.value = param.value.replace(&pattern, value);
        }
        
        // Replace in Body
        if let RequestBody::Raw { content, .. } = &mut config.body {
            *content = content.replace(&pattern, value);
        } else if let RequestBody::Json { value: json_val } = &mut config.body {
            let json_str = json_val.to_string().replace(&pattern, value);
            if let Ok(new_json) = serde_json::from_str(&json_str) {
                *json_val = new_json;
            }
        }
    }
}

/// Cancel an active HTTP request
#[allow(non_snake_case)]
#[tauri::command]
pub fn cancel_request(requestId: String) -> bool {
    http::cancel_request(&requestId)
}

/// Get request history
#[allow(non_snake_case)]
#[tauri::command]
pub async fn get_history(workspaceId: String, limit: i64, offset: i64) -> Result<serde_json::Value, String> {
    let history = db::get_history(&workspaceId, limit, offset)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(serde_json::to_value(history).map_err(|e| e.to_string())?)
}

/// Clear request history
#[allow(non_snake_case)]
#[tauri::command]
pub async fn clear_history(workspaceId: String) -> Result<(), String> {
    db::clear_history(&workspaceId)
        .await
        .map_err(|e| e.to_string())
}

/// Delete a single history entry
#[allow(non_snake_case)]
#[tauri::command]
pub async fn delete_history_entry(historyId: String) -> Result<(), String> {
    db::delete_history_entry(&historyId)
        .await
        .map_err(|e| e.to_string())
}

/// Get all settings
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

/// Update a setting
#[tauri::command]
pub async fn update_setting(key: String, value: String) -> Result<(), String> {
    // Update in database
    db::update_setting(&key, &value)
        .await
        .map_err(|e| e.to_string())?;

    // If max_redirects changed, rebuild HTTP executor
    if key == "max_redirects" {
        if let Ok(max_redirects) = value.parse::<usize>() {
            http::rebuild_executor(max_redirects).await;
        }
    }

    Ok(())
}

/// Create a new collection
#[allow(non_snake_case)]
#[tauri::command]
pub async fn create_collection(name: String, workspaceId: String) -> Result<String, String> {
    println!("Creating collection: {} in workspace: {}", name, workspaceId);
    let id = uuid::Uuid::new_v4().to_string();
    
    // Count existing collections to determine sort order
    let collections = db::get_collections(&workspaceId)
        .await
        .map_err(|e| e.to_string())?;
    let sort_order = collections.len() as i64;
    
    db::create_collection(&id, &workspaceId, &name, sort_order)
        .await
        .map_err(|e| e.to_string())?;
    trigger_push(workspaceId.clone());
    Ok(id)
}

/// Create a new folder
#[allow(non_snake_case)]
#[tauri::command]
pub async fn create_folder(name: String, collectionId: String) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    
    // Count existing folders to determine sort order
    let folders = db::get_folders(&collectionId)
        .await
        .map_err(|e| e.to_string())?;
    let sort_order = folders.len() as i64;
    
    db::create_folder(&id, &collectionId, &name, sort_order)
        .await
        .map_err(|e| e.to_string())?;
    if let Ok(Some(ws_id)) = db::get_workspace_id_by_collection_id(&collectionId).await {
        trigger_push(ws_id);
    }
    Ok(id)
}

/// Save a request
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
    let id = uuid::Uuid::new_v4().to_string();
    
    // Count existing requests to determine sort order
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
    if let Ok(Some(ws_id)) = db::get_workspace_id_by_collection_id(&collectionId).await {
        trigger_push(ws_id);
    }
    Ok(id)
}

/// Get collections with their folders and requests
#[allow(non_snake_case)]
#[tauri::command]
pub async fn get_collections_with_folders(workspaceId: String) -> Result<serde_json::Value, String> {
    let collections = db::get_collections(&workspaceId)
        .await
        .map_err(|e| e.to_string())?;
    
    let mut result = Vec::new();
    
    for (id, name, _) in collections {
        // Fetch folders for this collection
        let folders = db::get_folders(&id)
            .await
            .map_err(|e| e.to_string())?;
        
        // Fetch root requests for this collection (folder_id IS NULL)
        let all_requests = db::get_requests(&id)
            .await
            .map_err(|e| e.to_string())?;
        
        let mut folder_list = Vec::new();
        for (f_id, f_name, _) in folders {
            // Filter requests for this folder
            let folder_requests: Vec<serde_json::Value> = all_requests
                .iter()
                .filter(|(_, _, _, _, r_folder_id, _, _, _, _)| r_folder_id == &f_id)
                .map(|(r_id, r_name, r_method, r_url, _, r_headers, r_params, r_body, r_auth)| {
                    serde_json::json!({
                        "id": r_id,
                        "name": r_name,
                        "method": r_method,
                        "url": r_url,
                        "headers": serde_json::from_str::<serde_json::Value>(r_headers).unwrap_or(serde_json::json!([])),
                        "params": serde_json::from_str::<serde_json::Value>(r_params).unwrap_or(serde_json::json!([])),
                        "body": serde_json::from_str::<serde_json::Value>(r_body).unwrap_or(serde_json::json!({"type": "none", "content": ""})),
                        "auth": if r_auth.is_empty() {
                            serde_json::json!({"type": "none"})
                        } else {
                            serde_json::from_str::<serde_json::Value>(r_auth).unwrap_or(serde_json::json!({"type": "none"}))
                        }
                    })
                })
                .collect();

            folder_list.push(serde_json::json!({
                "id": f_id,
                "name": f_name,
                "requests": folder_requests
            }));
        }
        
        // Root requests (folder_id is empty or doesn't match any folder)
        let root_requests: Vec<serde_json::Value> = all_requests
            .into_iter()
            .filter(|(_, _, _, _, r_folder_id, _, _, _, _)| r_folder_id.is_empty())
            .map(|(r_id, r_name, r_method, r_url, _, r_headers, r_params, r_body, r_auth)| {
                serde_json::json!({
                    "id": r_id,
                    "name": r_name,
                    "method": r_method,
                    "url": r_url,
                    "headers": serde_json::from_str::<serde_json::Value>(&r_headers).unwrap_or(serde_json::json!([])),
                    "params": serde_json::from_str::<serde_json::Value>(&r_params).unwrap_or(serde_json::json!([])),
                    "body": serde_json::from_str::<serde_json::Value>(&r_body).unwrap_or(serde_json::json!({"type": "none", "content": ""})),
                    "auth": if r_auth.is_empty() {
                        serde_json::json!({"type": "none"})
                    } else {
                        serde_json::from_str::<serde_json::Value>(&r_auth).unwrap_or(serde_json::json!({"type": "none"}))
                    }
                })
            })
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

/// Ensure workspace exists
#[allow(non_snake_case)]
#[tauri::command]
pub async fn ensure_workspace(workspaceId: String) -> Result<String, String> {
    let id = db::ensure_workspace(&workspaceId)
        .await
        .map_err(|e| e.to_string())?;
    trigger_push(workspaceId.clone());
    Ok(id)
}

/// Create a new environment
#[allow(non_snake_case)]
#[tauri::command]
pub async fn create_environment(name: String, workspaceId: String) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    db::create_environment(&id, &workspaceId, &name)
        .await
        .map_err(|e| e.to_string())?;
    trigger_push(workspaceId.clone());
    Ok(id)
}

/// Get all environments in workspace
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

/// Set active environment
#[allow(non_snake_case)]
#[tauri::command]
pub async fn set_active_env(id: Option<String>, workspaceId: String) -> Result<(), String> {
    db::set_active_environment(id.as_deref(), &workspaceId)
        .await
        .map_err(|e| e.to_string())
}

/// Get variables for environment
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

/// Get active variables for workspace
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


/// Upsert multiple variables
#[allow(non_snake_case)]
#[tauri::command]
pub async fn save_variables(
    environmentId: String,
    vars: Vec<serde_json::Value>,
) -> Result<(), String> {
    for v in vars {
        let id = v["id"].as_str().unwrap_or(&uuid::Uuid::new_v4().to_string()).to_string();
        let key = v["key"].as_str().unwrap_or("");
        let value = v["value"].as_str().unwrap_or("");
        let is_secret = v["is_secret"].as_bool().unwrap_or(false);
        
        db::upsert_variable(&id, &environmentId, key, value, is_secret)
            .await
            .map_err(|e| e.to_string())?;
    }
    if let Ok(Some(ws_id)) = db::get_workspace_id_by_environment_id(&environmentId).await {
        trigger_push(ws_id);
    }
    Ok(())
}

#[allow(non_snake_case)]
#[tauri::command]
pub async fn delete_workspace(workspaceId: String) -> Result<(), String> {
    db::delete_workspace(&workspaceId)
        .await
        .map_err(|e| e.to_string())
}

#[allow(non_snake_case)]
#[tauri::command]
pub async fn delete_collection(collectionId: String) -> Result<(), String> {
    let ws_id = db::get_workspace_id_by_collection_id(&collectionId).await.ok().and_then(|x| x);
    db::delete_collection(&collectionId)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(id) = ws_id {
        trigger_push(id);
    }
    Ok(())
}

#[allow(non_snake_case)]
#[tauri::command]
pub async fn delete_folder(folderId: String) -> Result<(), String> {
    let ws_id = db::get_workspace_id_by_folder_id(&folderId).await.ok().and_then(|x| x);
    db::delete_folder(&folderId)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(id) = ws_id {
        trigger_push(id);
    }
    Ok(())
}

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
    let ws_id = db::get_workspace_id_by_request_id(&id).await.ok().and_then(|x| x);
    db::update_request(
        &id,
        &name,
        &method,
        &url,
        &headers,
        &params,
        &body,
        auth.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())?;
    if let Some(wid) = ws_id {
        trigger_push(wid);
    }
    Ok(())
}

#[allow(non_snake_case)]
#[tauri::command]
pub async fn delete_request(requestId: String) -> Result<(), String> {
    let ws_id = db::get_workspace_id_by_request_id(&requestId).await.ok().and_then(|x| x);
    db::delete_request(&requestId)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(id) = ws_id {
        trigger_push(id);
    }
    Ok(())
}

#[allow(non_snake_case)]
#[tauri::command]
pub async fn rename_workspace(workspaceId: String, name: String) -> Result<(), String> {
    db::update_workspace_name(&workspaceId, &name)
        .await
        .map_err(|e| e.to_string())?;
    trigger_push(workspaceId.clone());
    Ok(())
}

#[allow(non_snake_case)]
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

#[allow(non_snake_case)]
#[tauri::command]
pub async fn create_workspace(name: String) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    db::create_workspace(&id, &name)
        .await
        .map_err(|e| e.to_string())?;
    trigger_push(id.clone());
    Ok(id)
}

/// Batch import a collection with folders and requests
#[tauri::command]
pub async fn import_collection(collection: ImportCollection) -> Result<String, String> {
    let workspace_id = collection.workspace_id.clone();
    let id = db::import_collection(collection)
        .await
        .map_err(|e| e.to_string())?;
    trigger_push(workspace_id);
    Ok(id)
}

/// Clear all app data (workspaces, collections, history, environments, settings) and reset to defaults.
#[tauri::command]
pub async fn clear_all_data() -> Result<(), String> {
    db::clear_all_data().await.map_err(|e| e.to_string())
}

// --- Sync commands ---

#[tauri::command]
pub async fn get_sync_config() -> Result<serde_json::Value, String> {
    let config = sync::get_sync_config().await.map_err(|e| e.to_string())?;
    let url = match &config {
        Some((u, _)) => u.clone(),
        None => db::get_setting("sync_server_url").await.ok().and_then(|s| s).map(|s| s.value).unwrap_or_default(),
    };
    let has_token = config.is_some();
    let last_error = db::get_setting("sync_last_error").await.ok().and_then(|s| s).filter(|s| !s.value.is_empty()).map(|s| s.value).unwrap_or_default();
    let last_synced_at: Option<i64> = db::get_setting("sync_last_synced_at").await.ok().and_then(|s| s).and_then(|s| s.value.parse().ok());
    Ok(serde_json::json!({
        "url": url,
        "hasToken": has_token,
        "lastError": if last_error.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(last_error) },
        "lastSyncedAt": last_synced_at
    }))
}

#[tauri::command]
pub async fn set_sync_config(url: Option<String>, token: Option<String>) -> Result<(), String> {
    sync::set_sync_config(url, token).await
}

#[allow(non_snake_case)]
#[tauri::command]
pub async fn sync_login(syncUrl: String, email: String, password: String) -> Result<serde_json::Value, String> {
    let base = syncUrl.trim_end_matches('/');
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/auth/login", base))
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Login failed: {} {}", status, body));
    }
    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let token = json.get("token").and_then(|v| v.as_str()).ok_or("No token in response")?;
    sync::set_sync_config(Some(syncUrl.clone()), Some(token.to_string())).await?;
    Ok(serde_json::json!({ "email": json.get("email").and_then(|v| v.as_str()).unwrap_or("") }))
}

#[allow(non_snake_case)]
#[tauri::command]
pub async fn sync_register(syncUrl: String, email: String, password: String) -> Result<serde_json::Value, String> {
    let base = syncUrl.trim_end_matches('/');
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/auth/register", base))
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Register failed: {} {}", status, body));
    }
    let _: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    sync_login(syncUrl, email, password).await
}

#[tauri::command]
pub async fn sync_logout() -> Result<(), String> {
    sync::set_sync_config(None, Some(String::new())).await?;
    Ok(())
}

#[tauri::command]
pub async fn clear_sync_error() -> Result<(), String> {
    sync::clear_sync_error().await
}

#[allow(non_snake_case)]
#[tauri::command]
pub async fn pull_workspace(workspaceId: String) -> Result<(), String> {
    sync::pull_workspace(&workspaceId).await
}

#[allow(non_snake_case)]
#[tauri::command]
pub async fn invite_to_workspace(workspaceId: String, email: String) -> Result<String, String> {
    sync::invite_to_workspace(&workspaceId, &email).await
}


