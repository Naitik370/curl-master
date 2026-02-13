use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT, ACCEPT};
use keyring::Entry;
use base64::{Engine as _, engine::general_purpose};

const APP_NAME: &str = "curlmaster";
const GITHUB_TOKEN_KEY: &str = "github_pat";

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubFileResponse {
    pub sha: String,
    pub content: String,
    pub encoding: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubUpdateResponse {
    pub content: serde_json::Value,
    pub commit: serde_json::Value,
}

pub struct GithubClient {
    client: reqwest::Client,
}

impl GithubClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn store_token(&self, token: &str) -> Result<(), String> {
        let entry = Entry::new(APP_NAME, GITHUB_TOKEN_KEY).map_err(|e| e.to_string())?;
        entry.set_password(token).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_token(&self) -> Result<String, String> {
        let entry = Entry::new(APP_NAME, GITHUB_TOKEN_KEY).map_err(|e| e.to_string())?;
        entry.get_password().map_err(|e| e.to_string())
    }

    pub fn delete_token(&self) -> Result<(), String> {
        let entry = Entry::new(APP_NAME, GITHUB_TOKEN_KEY).map_err(|e| e.to_string())?;
        entry.delete_password().map_err(|e| e.to_string())
    }

    async fn get_headers(&self) -> Result<HeaderMap, String> {
        let token = self.get_token()?;
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token)).map_err(|e| e.to_string())?);
        headers.insert(USER_AGENT, HeaderValue::from_static(APP_NAME));
        headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github.v3+json"));
        Ok(headers)
    }

    pub async fn repo_exists(&self, repo: &str) -> Result<bool, String> {
        let url = format!("https://api.github.com/repos/{}", repo);
        let headers = self.get_headers().await?;

        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        Ok(response.status().is_success())
    }

    pub async fn fetch_file(&self, repo: &str, path: &str, branch: &str) -> Result<Option<GithubFileResponse>, String> {
        let url = format!("https://api.github.com/repos/{}/contents/{}?ref={}", repo, path, branch);
        let headers = self.get_headers().await?;

        let response = self.client.get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            // Check if repo exists to give a better error
            if !self.repo_exists(repo).await.unwrap_or(true) {
                return Err(format!("Repository '{}' not found. Please make sure it exists and your token has access.", repo));
            }
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("GitHub API error ({}): {}", status, error_text));
        }

        let res = response.json::<GithubFileResponse>().await.map_err(|e| e.to_string())?;
        Ok(Some(res))
    }

    pub async fn push_file(&self, repo: &str, path: &str, branch: &str, content: &str, message: &str, sha: Option<&str>) -> Result<GithubUpdateResponse, String> {
        let url = format!("https://api.github.com/repos/{}/contents/{}", repo, path);
        let headers = self.get_headers().await?;

        let encoded_content = general_purpose::STANDARD.encode(content);

        let mut body = serde_json::json!({
            "message": message,
            "content": encoded_content,
            "branch": branch,
        });

        if let Some(s) = sha {
            body["sha"] = serde_json::Value::String(s.to_string());
        }

        let response = self.client.put(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("GitHub API error: {}", error_text));
        }

        response.json::<GithubUpdateResponse>().await.map_err(|e| e.to_string())
    }
}
