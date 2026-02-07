use serde::{Deserialize, Serialize};

/// Key-value pair with enable/disable toggle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

/// Request body types with tagged enum to prevent invalid states
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RequestBody {
    None,
    Raw { 
        mime: String, 
        content: String 
    },
    Json { 
        value: serde_json::Value 
    },
    FormUrlEncoded { 
        fields: Vec<KeyValue> 
    },
    Multipart { 
        fields: Vec<KeyValue> 
    },
}

/// HTTP authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthConfig {
    None,
    Basic { 
        username: String, 
        password: String 
    },
    Bearer { 
        token: String 
    },
}

/// Request configuration sent from frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestConfig {
    pub method: String,
    pub url: String,
    pub headers: Vec<KeyValue>,
    pub params: Vec<KeyValue>,
    pub body: RequestBody,
    pub auth: Option<AuthConfig>,
    pub timeout_ms: u64,
    pub ignore_tls: bool,
    pub follow_redirects: bool,
}

/// Response body with different variants for safe handling
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseBody {
    Text { 
        content: String 
    },
    Binary { 
        preview_hex: String 
    },
    Truncated { 
        content: String, 
        original_size: usize 
    },
}

/// HTTP response returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<KeyValue>,
    pub body: ResponseBody,
    pub size_bytes: usize,
    pub time_ms: u128,
    pub content_type: Option<String>,
    pub content_encoding: Option<String>,
}

/// Transport-level errors (4xx/5xx are NOT errors, they're valid responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HttpError {
    Network { message: String },
    Timeout,
    Dns { message: String },
    Tls { message: String },
    InvalidUrl { message: String },
    Cancelled,
}

/// Tagged enum result - no invalid states
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum SendResult {
    Success { 
        request_id: String, 
        response: HttpResponse 
    },
    Failed { 
        request_id: String, 
        error: HttpError 
    },
}

/// Database models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub collection_id: String,
    pub parent_folder_id: Option<String>,
    pub name: String,
    pub sort_order: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub folder_id: Option<String>,
    pub collection_id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: String,  // JSON
    pub params: String,   // JSON
    pub body: String,     // JSON
    pub auth: Option<String>,  // JSON
    pub sort_order: i64,
    pub schema_version: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: String,
    pub workspace_id: String,
    pub request_id: Option<String>,
    pub method: String,
    pub url: String,
    pub request_headers: String,
    pub request_params: String,
    pub request_body: Option<String>,
    pub response_status: u16,
    pub response_headers: String,
    pub response_time_ms: u64,
    pub response_size_bytes: usize,
    pub created_at: i64,
}

// Import models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRequest {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: String,
    pub params: String,
    pub body: String,
    pub auth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportFolder {
    pub name: String,
    pub requests: Vec<ImportRequest>,
    pub folders: Vec<ImportFolder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCollection {
    pub name: String,
    pub workspace_id: String,
    pub folders: Vec<ImportFolder>,
    pub requests: Vec<ImportRequest>,
}
