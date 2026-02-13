use crate::{db, http, models::*};

// ---------------------------------------------------------------------------
// HTTP request execution
// ---------------------------------------------------------------------------

/// Send HTTP request with variable substitution.
///
/// Steps:
/// 1. Fetch active variables for the workspace
/// 2. Substitute `{{var}}` patterns in URLs, headers, params, and body
/// 3. Execute the request
/// 4. Persist a history entry on success
#[allow(non_snake_case)]
#[tauri::command]
pub async fn send_request(mut config: RequestConfig, workspaceId: String) -> SendResult {
    // 1. Fetch active variables for this workspace
    let variables = match db::get_active_variables(&workspaceId).await {
        Ok(v) => v,
        Err(e) => {
            return SendResult::Failed {
                request_id: "error".to_string(),
                error: HttpError::Network {
                    message: format!("Failed to fetch variables: {}", e),
                },
            };
        }
    };

    // 2. Perform substitution in URL, headers, params, and body
    substitute_variables(&mut config, &variables);

    // 3. Execute request
    let result = http::execute_request(config.clone()).await;

    // 4. Save to history if successful
    if let SendResult::Success { response, .. } = &result {
        save_history_entry(&workspaceId, &config, response).await;
    }

    result
}

/// Cancel an active HTTP request by its tracking ID.
#[allow(non_snake_case)]
#[tauri::command]
pub fn cancel_request(requestId: String) -> bool {
    http::cancel_request(&requestId)
}

// ---------------------------------------------------------------------------
// Variable substitution
// ---------------------------------------------------------------------------

/// Replace every `{{key}}` occurrence in request config fields.
fn substitute_variables(config: &mut RequestConfig, variables: &[(String, String)]) {
    for (key, value) in variables {
        let pattern = format!("{{{{{}}}}}", key);

        // URL
        config.url = config.url.replace(&pattern, value);

        // Headers
        for header in &mut config.headers {
            header.key = header.key.replace(&pattern, value);
            header.value = header.value.replace(&pattern, value);
        }

        // Query params
        for param in &mut config.params {
            param.key = param.key.replace(&pattern, value);
            param.value = param.value.replace(&pattern, value);
        }

        // Body (only Raw and Json variants contain substitutable text)
        match &mut config.body {
            RequestBody::Raw { content, .. } => {
                *content = content.replace(&pattern, value);
            }
            RequestBody::Json { value: json_val } => {
                let json_str = json_val.to_string().replace(&pattern, value);
                if let Ok(new_json) = serde_json::from_str(&json_str) {
                    *json_val = new_json;
                }
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// History persistence (private helper)
// ---------------------------------------------------------------------------

/// Serialize request/response data and persist a history entry.
///
/// Extracted from `send_request` to keep it focused (SRP).
async fn save_history_entry(
    workspace_id: &str,
    config: &RequestConfig,
    response: &HttpResponse,
) {
    let req_headers = serde_json::to_string(&config.headers).unwrap_or_default();
    let req_params = serde_json::to_string(&config.params).unwrap_or_default();

    let req_body = match &config.body {
        RequestBody::None => None,
        RequestBody::Raw { content, .. } => Some(content.clone()),
        RequestBody::Json { value } => Some(value.to_string()),
        RequestBody::FormUrlEncoded { fields } => {
            Some(serde_json::to_string(fields).unwrap_or_default())
        }
        RequestBody::Multipart { fields } => {
            Some(serde_json::to_string(fields).unwrap_or_default())
        }
    };

    let res_headers = serde_json::to_string(&response.headers).unwrap_or_default();

    let (res_body_full, res_body_preview, res_body_type) = extract_response_body(&response.body);

    if let Err(e) = db::add_history_entry(
        &uuid::Uuid::new_v4().to_string(),
        workspace_id,
        None,
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
    )
    .await
    {
        eprintln!("Failed to save history: {:?}", e);
    }
}

/// Extract full body, preview (first 1000 chars), and type label from a response body.
fn extract_response_body(
    body: &ResponseBody,
) -> (Option<String>, Option<String>, &'static str) {
    match body {
        ResponseBody::Text { content } => {
            let preview: String = content.chars().take(1000).collect();
            (Some(content.clone()), Some(preview), "text")
        }
        ResponseBody::Binary { preview_hex } => {
            (None, Some(preview_hex.clone()), "binary")
        }
        ResponseBody::Truncated { content, .. } => {
            let preview: String = content.chars().take(1000).collect();
            (Some(content.clone()), Some(preview), "truncated")
        }
    }
}
