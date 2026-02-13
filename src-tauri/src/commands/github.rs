use crate::db;
use base64::Engine;

// ---------------------------------------------------------------------------
// GitHub sync commands
// ---------------------------------------------------------------------------

/// Get the stored GitHub personal access token.
#[tauri::command]
pub async fn get_github_token() -> Result<String, String> {
    crate::github::GithubClient::new().get_token()
}

/// Store a GitHub personal access token in the system keyring.
#[tauri::command]
pub async fn save_github_token(token: String) -> Result<(), String> {
    crate::github::GithubClient::new().store_token(&token)
}

/// Delete the stored GitHub personal access token.
#[tauri::command]
pub async fn delete_github_token() -> Result<(), String> {
    crate::github::GithubClient::new().delete_token()
}

/// Push the current workspace data to a GitHub repository file.
///
/// Inputs are sanitized (trimmed, leading/trailing slashes removed).
#[tauri::command]
pub async fn sync_to_github(
    workspace_id: String,
    repo: String,
    path: String,
    branch: String,
) -> Result<(), String> {
    let client = crate::github::GithubClient::new();
    let (repo, path, branch) = sanitize_github_inputs(repo, path, branch);

    // 1. Export workspace data
    let data = db::export_workspace(&workspace_id)
        .await
        .map_err(|e| e.to_string())?;

    let content = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;

    // 2. Get current SHA if file already exists (needed for update)
    let sha = client
        .fetch_file(&repo, &path, &branch)
        .await?
        .map(|res| res.sha);

    // 3. Push to GitHub
    client
        .push_file(
            &repo,
            &path,
            &branch,
            &content,
            &format!("Sync from CurlMaster - {}", chrono::Utc::now()),
            sha.as_deref(),
        )
        .await?;

    Ok(())
}

/// Pull workspace data from a GitHub repository file.
///
/// Returns the parsed JSON for the frontend to decide merge vs. overwrite.
#[tauri::command]
pub async fn sync_from_github(
    _workspace_id: String,
    repo: String,
    path: String,
    branch: String,
) -> Result<serde_json::Value, String> {
    let client = crate::github::GithubClient::new();
    let (repo, path, branch) = sanitize_github_inputs(repo, path, branch);

    // 1. Fetch from GitHub
    let res = client
        .fetch_file(&repo, &path, &branch)
        .await?
        .ok_or_else(|| {
            format!(
                "The sync file '{}' was not found in the repository '{}'.",
                path, repo
            )
        })?;

    // 2. Decode base64 content
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(res.content.replace('\n', "").replace('\r', ""))
        .map_err(|e| e.to_string())?;

    let decoded_str = String::from_utf8(decoded_bytes).map_err(|e| e.to_string())?;

    // 3. Parse and return JSON
    serde_json::from_str(&decoded_str).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Sanitize and normalize GitHub-related user inputs.
///
/// - Trims whitespace
/// - Strips leading/trailing slashes from repo and path
/// - Defaults branch to "main" if empty
fn sanitize_github_inputs(repo: String, path: String, branch: String) -> (String, String, String) {
    let repo = repo
        .trim()
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string();
    let path = path.trim().trim_start_matches('/').to_string();
    let branch = {
        let b = branch.trim().to_string();
        if b.is_empty() { "main".to_string() } else { b }
    };

    (repo, path, branch)
}
