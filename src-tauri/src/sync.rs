//! Sync client: push/pull workspace snapshots to/from the sync server.

use crate::db;
use crate::models::WorkspaceSnapshot;
use reqwest::Client;

const SETTING_SYNC_URL: &str = "sync_server_url";
const SETTING_SYNC_TOKEN: &str = "sync_token";
const SETTING_SYNC_LAST_ERROR: &str = "sync_last_error";
const SETTING_SYNC_LAST_AT: &str = "sync_last_synced_at";

/// Get sync server URL and token from settings. Returns None if not configured.
pub async fn get_sync_config() -> Result<Option<(String, String)>, String> {
    let url = db::get_setting(SETTING_SYNC_URL).await.map_err(|e| e.to_string())?;
    let token = db::get_setting(SETTING_SYNC_TOKEN).await.map_err(|e| e.to_string())?;
    match (url, token) {
        (Some(u), Some(t)) if !u.value.is_empty() && !t.value.is_empty() => Ok(Some((u.value, t.value))),
        _ => Ok(None),
    }
}

pub async fn set_sync_config(url: Option<String>, token: Option<String>) -> Result<(), String> {
    if let Some(u) = url {
        db::update_setting(SETTING_SYNC_URL, &u).await.map_err(|e| e.to_string())?;
    }
    if let Some(t) = token {
        db::update_setting(SETTING_SYNC_TOKEN, &t).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn clear_sync_error() -> Result<(), String> {
    db::update_setting(SETTING_SYNC_LAST_ERROR, "").await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Push workspace snapshot to server. Fails silently if not configured or on network error (returns Err with message).
pub async fn push_workspace(workspace_id: &str) -> Result<(), String> {
    let (base_url, token) = match get_sync_config().await? {
        Some(c) => c,
        None => return Err("Sync not configured".to_string()),
    };
    let url = format!("{}/sync/workspaces/{}", base_url.trim_end_matches('/'), workspace_id);
    let snapshot = db::get_workspace_snapshot(workspace_id)
        .await
        .map_err(|e| e.to_string())?;
    let client = Client::new();
    let res = client
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&snapshot)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Sync failed: {} {}", status, body));
    }
    Ok(())
}

/// Invite a user to workspace by email.
pub async fn invite_to_workspace(workspace_id: &str, email: &str) -> Result<String, String> {
    let (base_url, token) = match get_sync_config().await? {
        Some(c) => c,
        None => return Err("Sync not configured".to_string()),
    };
    let url = format!("{}/workspaces/{}/invite", base_url.trim_end_matches('/'), workspace_id);
    let client = Client::new();
    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "email": email }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Invite failed: {} {}", status, body));
    }
    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    let link = json
        .get("inviteLink")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok(link)
}

/// Pull workspace snapshot from server and replace local data.
pub async fn pull_workspace(workspace_id: &str) -> Result<(), String> {
    let (base_url, token) = match get_sync_config().await? {
        Some(c) => c,
        None => return Err("Sync not configured".to_string()),
    };
    let url = format!("{}/sync/workspaces/{}", base_url.trim_end_matches('/'), workspace_id);
    let client = Client::new();
    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("Sync failed: {} {}", status, body));
    }
    let snapshot: WorkspaceSnapshot = res.json().await.map_err(|e| e.to_string())?;
    db::replace_workspace_from_snapshot(&snapshot)
        .await
        .map_err(|e| e.to_string())?;
    let _ = db::update_setting(SETTING_SYNC_LAST_AT, &chrono::Utc::now().timestamp_millis().to_string()).await;
    let _ = db::update_setting(SETTING_SYNC_LAST_ERROR, "").await;
    Ok(())
}
