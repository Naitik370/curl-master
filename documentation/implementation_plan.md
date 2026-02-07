# CurlMaster - Phase 1 Implementation Plan (Production Ready)

Desktop API client: **Tauri + Vue 3 + SQLite**

## Decisions

| Item | Choice |
|------|--------|
| App Name | CurlMaster |
| Window | Custom titlebar |
| Import/Export | CurlMaster + Postman |
| Timeline | 5-6 weeks |

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| Runtime | Tauri 2.x |
| Frontend | Vue 3 + Vite + Pinia |
| HTTP | reqwest (Rust) - Quad clients |
| Database | sqlx + SQLite |

---

## Phase 1 Scope

**Included:** Request builder, response viewer, collections/folders, environments, code snippets, import/export, TLS toggle, timeout, cancel, redirects

**Excluded:** File upload, OAuth, global variables, scripts, cloud sync, **proxy** (schema placeholder only)

---

## Critical Architecture

### 1. Quad HTTP Clients
```rust
pub struct HttpExecutor {
    strict_follow: reqwest::Client,
    strict_nofollow: reqwest::Client,
    insecure_follow: reqwest::Client,
    insecure_nofollow: reqwest::Client,
}

impl HttpExecutor {
    pub fn new() -> Self {
        Self {
            strict_follow: build_client(false, true),
            strict_nofollow: build_client(false, false),
            insecure_follow: build_client(true, true),
            insecure_nofollow: build_client(true, false),
        }
    }
    
    fn select_client(&self, config: &RequestConfig) -> &reqwest::Client {
        match (config.ignore_tls, config.follow_redirects) {
            (false, true) => &self.strict_follow,
            (false, false) => &self.strict_nofollow,
            (true, true) => &self.insecure_follow,
            (true, false) => &self.insecure_nofollow,
        }
    }
}

// max_redirects is a GLOBAL SETTING (from settings table)
fn build_client(insecure: bool, follow: bool, max_redirects: usize) -> reqwest::Client {
    let policy = if follow { 
        Policy::limited(max_redirects) 
    } else { 
        Policy::none() 
    };
    reqwest::Client::builder()
        .danger_accept_invalid_certs(insecure)
        .redirect(policy)
        .build().unwrap()
}

// Rebuild clients when max_redirects setting changes
// Thread-safe executor swap via Arc<RwLock<HttpExecutor>>
```

### Thread-Safe Executor (for runtime setting changes)
```rust
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static! {
    static ref EXECUTOR: Arc<RwLock<HttpExecutor>> = Arc::new(RwLock::new(HttpExecutor::new(10)));
}

// Call when max_redirects changes in settings
async fn rebuild_executor(max_redirects: usize) {
    let mut executor = EXECUTOR.write().await;
    *executor = HttpExecutor::new(max_redirects);
}

### 2. True Request Abort + SendResult
```rust
use futures::future::{Abortable, AbortHandle};
use dashmap::DashMap;

lazy_static! {
    static ref ACTIVE_REQUESTS: DashMap<String, AbortHandle> = DashMap::new();
}

// Tagged enum - no invalid states
#[derive(Serialize)]
#[serde(tag = "status")]
pub enum SendResult {
    Success { request_id: String, response: HttpResponse },
    Failed { request_id: String, error: HttpError },
}

#[tauri::command]
async fn send_request(config: RequestConfig) -> SendResult {
    let request_id = uuid::Uuid::new_v4().to_string();
    let (abort_handle, abort_reg) = AbortHandle::new_pair();
    
    ACTIVE_REQUESTS.insert(request_id.clone(), abort_handle);
    
    let executor = EXECUTOR.read().await;
    let result = tokio::time::timeout(
        Duration::from_millis(config.timeout_ms),
        Abortable::new(executor.execute(config), abort_reg)
    ).await;
    
    ACTIVE_REQUESTS.remove(&request_id);
    
    match result {
        Ok(Ok(Ok(resp))) => SendResult::Success { request_id, response: resp },
        Ok(Ok(Err(e))) => SendResult::Failed { request_id, error: e },
        Ok(Err(_)) => SendResult::Failed { request_id, error: HttpError::Cancelled },
        Err(_) => SendResult::Failed { request_id, error: HttpError::Timeout },
    }
}

#[tauri::command]
fn cancel_request(request_id: String) {
    if let Some((_, handle)) = ACTIVE_REQUESTS.remove(&request_id) {
        handle.abort();
    }
}

// Implementation note: executor.execute() must NOT spawn tasks
// Use: client.request().send().await then resp.bytes().await
// Both are cancellable when abort drops the Abortable future
```

### 3. Error Model (Transport-Only)
```rust
pub enum HttpError {
    Network(String),
    Timeout,
    Dns(String),
    Tls(String),
    InvalidUrl(String),
    Cancelled,
}
// 4xx/5xx → returned as HttpResponse, NOT errors
```

### 4. Typed Request Body
```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RequestBody {
    None,
    Raw { mime: String, content: String },
    Json { value: serde_json::Value },
    FormUrlEncoded { fields: Vec<KeyValue> },
    Multipart { fields: Vec<KeyValue> },
}
```

### 5. Headers/Params as Vec
```rust
pub struct KeyValue {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

// Merge order at execution: system → auth → user (user wins)
```

### 6. Safe Response Body
```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseBody {
    Text { content: String },
    Binary { preview_hex: String },
    Truncated { content: String, original_size: usize },
}
```

### 7. Variable Substitution
```
Escape syntax: \{{literal}} → renders as {{literal}}

Frontend: highlighting + unresolved detection only
Rust: authoritative substitution before execution
```

---

## Database Schema

```sql
-- CRITICAL: Enable foreign key enforcement
PRAGMA foreign_keys = ON;
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

CREATE TABLE workspace (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE collection (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspace(id),
    name TEXT NOT NULL,
    description TEXT,
    sort_order INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE folder (
    id TEXT PRIMARY KEY,
    collection_id TEXT NOT NULL REFERENCES collection(id),
    parent_folder_id TEXT REFERENCES folder(id),
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

-- folder_id NULL = collection root
CREATE TABLE request (
    id TEXT PRIMARY KEY,
    folder_id TEXT REFERENCES folder(id),
    collection_id TEXT NOT NULL REFERENCES collection(id),
    name TEXT NOT NULL,
    method TEXT NOT NULL,
    url TEXT NOT NULL,
    headers TEXT NOT NULL,  -- JSON
    params TEXT NOT NULL,   -- JSON
    body TEXT NOT NULL,     -- JSON
    auth TEXT,              -- JSON
    sort_order INTEGER NOT NULL,
    schema_version INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL
);

CREATE TABLE environment (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspace(id),
    name TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE variable (
    id TEXT PRIMARY KEY,
    environment_id TEXT NOT NULL REFERENCES environment(id),
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    is_secret INTEGER NOT NULL DEFAULT 0
);

-- Split response storage (workspace_id for pruning)
CREATE TABLE history (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspace(id),
    request_id TEXT REFERENCES request(id),
    method TEXT NOT NULL,
    url TEXT NOT NULL,
    request_headers TEXT NOT NULL,
    request_body TEXT,
    response_status INTEGER NOT NULL,
    response_headers TEXT NOT NULL,
    response_body_preview TEXT,
    response_body_full TEXT,
    response_body_type TEXT NOT NULL,
    response_size_bytes INTEGER NOT NULL,
    response_time_ms INTEGER NOT NULL,
    content_type TEXT,
    content_encoding TEXT,
    created_at INTEGER NOT NULL
);

CREATE TABLE tab_state (
    id TEXT PRIMARY KEY,
    request_id TEXT REFERENCES request(id),
    draft_data TEXT NOT NULL,  -- JSON
    draft_version INTEGER NOT NULL DEFAULT 1,
    tab_order INTEGER NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0
);
```

---

## Constants
```rust
const HISTORY_PREVIEW_LIMIT: usize = 256 * 1024;  // 256KB
const HISTORY_FULL_LIMIT: usize = 256 * 1024;     // Store full only if <= this
const HISTORY_MAX_ENTRIES: usize = 100;           // Per workspace
const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_MAX_REDIRECTS: usize = 10;
```

## Settings Keys
```
timeout_ms, max_redirects, theme,
ignore_tls_default, follow_redirects_default,
proxy (JSON), history_limit
```

## UTF-8 Detection
```
If content-type starts with text/ OR contains json/xml/javascript:
  → attempt UTF-8 decode
  → if decode fails → treat as binary
Else → binary
```

## Invariants

1. `folder_id = NULL` → request at collection root
2. `collection_id` must match `folder.collection_id` (enforce in code)
3. Prune history to 100 entries per workspace
4. Sort: new = max + 1000, reorder = (prev + next) / 2

---

## Postman Import
```typescript
interface ImportResult {
  collection: CurlMasterCollection;
  warnings: string[];
  ignoredFields: string[];
  stats: { requests: number; folders: number; scriptsIgnored: number };
}
```

---

## Verification Checklist

- [ ] HTTP: GET, POST JSON, POST form, headers, params, auth types
- [ ] TLS ignore (badssl.com), timeout, cancel mid-flight
- [ ] Large response truncation, binary detection, non-UTF8
- [ ] Redirects toggle, duplicate headers
- [ ] Variable substitution (URL, headers, body), escape syntax
- [ ] Collections CRUD, folders, drag-drop, import/export
- [ ] Tab persistence across restart
- [ ] Postman import with warnings display

---

## Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+Enter` | Send |
| `Ctrl+N` | New tab |
| `Ctrl+W` | Close tab |
| `Ctrl+S` | Save |
| `Ctrl+.` | Cancel |
