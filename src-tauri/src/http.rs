use crate::models::*;
use dashmap::DashMap;
use futures::future::{AbortHandle, Abortable};
use lazy_static::lazy_static;
use reqwest::{redirect::Policy, Client, Method};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// Constants from plan
const RESPONSE_PREVIEW_LIMIT: usize = 256 * 1024; // 256KB

lazy_static! {
    /// Global HTTP executor with thread-safe access for runtime reconfiguration
    static ref EXECUTOR: Arc<RwLock<HttpExecutor>> = Arc::new(RwLock::new(HttpExecutor::new(10)));
    
    /// Active requests tracker for cancellation
    static ref ACTIVE_REQUESTS: DashMap<String, AbortHandle> = DashMap::new();
}

/// Quad HTTP clients for all TLS/redirect combinations
pub struct HttpExecutor {
    strict_follow: Client,
    strict_nofollow: Client,
    insecure_follow: Client,
    insecure_nofollow: Client,
}

impl HttpExecutor {
    /// Create new executor with specified max redirects
    pub fn new(max_redirects: usize) -> Self {
        Self {
            strict_follow: build_client(false, true, max_redirects),
            strict_nofollow: build_client(false, false, max_redirects),
            insecure_follow: build_client(true, true, max_redirects),
            insecure_nofollow: build_client(true, false, max_redirects),
        }
    }

    /// Select the appropriate client based on config
    fn select_client(&self, config: &RequestConfig) -> &Client {
        match (config.ignore_tls, config.follow_redirects) {
            (false, true) => &self.strict_follow,
            (false, false) => &self.strict_nofollow,
            (true, true) => &self.insecure_follow,
            (true, false) => &self.insecure_nofollow,
        }
    }

    /// Execute HTTP request
    pub async fn execute(&self, config: RequestConfig) -> Result<HttpResponse, HttpError> {
        let start = Instant::now();
        
        // Parse method
        let method = Method::from_bytes(config.method.as_bytes())
            .map_err(|_| HttpError::InvalidUrl { 
                message: "Invalid HTTP method".to_string() 
            })?;

        // Select appropriate client
        let client = self.select_client(&config);

        // Build request
        let mut request = client.request(method, &config.url);

        // Add query params (only enabled ones)
        for param in config.params.iter().filter(|p| p.enabled) {
            request = request.query(&[(param.key.as_str(), param.value.as_str())]);
        }

        // Add headers (only enabled ones)
        for header in config.headers.iter().filter(|h| h.enabled) {
            request = request.header(header.key.as_str(), header.value.as_str());
        }

        // Apply authentication
        if let Some(auth) = &config.auth {
            request = match auth {
                AuthConfig::Basic { username, password } => {
                    request.basic_auth(username, Some(password))
                }
                AuthConfig::Bearer { token } => {
                    request.bearer_auth(token)
                }
                AuthConfig::None => request,
            };
        }

        // Add body
        request = match &config.body {
            RequestBody::None => request,
            RequestBody::Raw { mime, content } => {
                request
                    .header("Content-Type", mime)
                    .body(content.clone())
            }
            RequestBody::Json { value } => {
                request.json(value)
            }
            RequestBody::FormUrlEncoded { fields } => {
                let form_data: Vec<(String, String)> = fields
                    .iter()
                    .filter(|f| f.enabled)
                    .map(|f| (f.key.clone(), f.value.clone()))
                    .collect();
                request.form(&form_data)
            }
            RequestBody::Multipart { fields } => {
                let mut form = reqwest::multipart::Form::new();
                for field in fields.iter().filter(|f| f.enabled) {
                    form = form.text(field.key.clone(), field.value.clone());
                }
                request.multipart(form)
            }
        };

        // Send request (cancellable)
        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                HttpError::Timeout
            } else if e.is_connect() {
                HttpError::Network { message: e.to_string() }
            } else if e.is_request() {
                HttpError::InvalidUrl { message: e.to_string() }
            } else {
                HttpError::Network { message: e.to_string() }
            }
        })?;

        // Extract response metadata
        let status = response.status().as_u16();
        let status_text = response.status().canonical_reason()
            .unwrap_or("Unknown")
            .to_string();

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let content_encoding = response
            .headers()
            .get("content-encoding")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Convert headers to KeyValue
        let headers: Vec<KeyValue> = response
            .headers()
            .iter()
            .map(|(k, v)| KeyValue {
                key: k.to_string(),
                value: v.to_str().unwrap_or("").to_string(),
                enabled: true,
            })
            .collect();

        // Read body bytes (cancellable)
        let bytes = response.bytes().await.map_err(|e| HttpError::Network { 
            message: e.to_string() 
        })?;

        let size_bytes = bytes.len();
        let time_ms = start.elapsed().as_millis();

        // Determine if content is text-based
        let is_text = content_type
            .as_ref()
            .map(|ct| {
                ct.starts_with("text/") 
                || ct.contains("json") 
                || ct.contains("xml") 
                || ct.contains("javascript")
            })
            .unwrap_or(false);

        // Parse body based on type and size
        let body = if is_text {
            match String::from_utf8(bytes.to_vec()) {
                Ok(text) => {
                    if text.len() > RESPONSE_PREVIEW_LIMIT {
                        ResponseBody::Truncated {
                            content: text[..RESPONSE_PREVIEW_LIMIT].to_string(),
                            original_size: text.len(),
                        }
                    } else {
                        ResponseBody::Text { content: text }
                    }
                }
                Err(_) => {
                    // Failed UTF-8 decode, treat as binary
                    let preview = bytes
                        .iter()
                        .take(256)
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>();
                    ResponseBody::Binary { preview_hex: preview }
                }
            }
        } else {
            // Binary content
            let preview = bytes
                .iter()
                .take(256)
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
            ResponseBody::Binary { preview_hex: preview }
        };

        Ok(HttpResponse {
            status,
            status_text,
            headers,
            body,
            size_bytes,
            time_ms,
            content_type,
            content_encoding,
        })
    }
}

/// Build reqwest client with specified configuration
fn build_client(insecure: bool, follow: bool, max_redirects: usize) -> Client {
    let policy = if follow {
        Policy::limited(max_redirects)
    } else {
        Policy::none()
    };

    Client::builder()
        .danger_accept_invalid_certs(insecure)
        .redirect(policy)
        .build()
        .expect("Failed to build HTTP client")
}

/// Rebuild executor when settings change
pub async fn rebuild_executor(max_redirects: usize) {
    let mut executor = EXECUTOR.write().await;
    *executor = HttpExecutor::new(max_redirects);
}

/// Execute request with timeout and cancellation support
pub async fn execute_request(config: RequestConfig) -> SendResult {
    let request_id = uuid::Uuid::new_v4().to_string();
    let (abort_handle, abort_reg) = AbortHandle::new_pair();

    ACTIVE_REQUESTS.insert(request_id.clone(), abort_handle);

    let executor = EXECUTOR.read().await;
    let timeout_duration = Duration::from_millis(config.timeout_ms);

    let result = tokio::time::timeout(
        timeout_duration,
        Abortable::new(executor.execute(config), abort_reg),
    )
    .await;

    ACTIVE_REQUESTS.remove(&request_id);

    match result {
        Ok(Ok(Ok(response))) => SendResult::Success {
            request_id,
            response,
        },
        Ok(Ok(Err(error))) => SendResult::Failed {
            request_id,
            error,
        },
        Ok(Err(_)) => SendResult::Failed {
            request_id,
            error: HttpError::Cancelled,
        },
        Err(_) => SendResult::Failed {
            request_id,
            error: HttpError::Timeout,
        },
    }
}

/// Cancel an active request
pub fn cancel_request(request_id: &str) -> bool {
    if let Some((_, handle)) = ACTIVE_REQUESTS.remove(request_id) {
        handle.abort();
        true
    } else {
        false
    }
}
