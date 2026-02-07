# Development Guide

## Quick Start

### Running the App

```bash
npm run tauri dev
```

This will:
1. Start the Vite dev server on http://localhost:5173
2. Compile the Rust backend
3. Open the application window

### Development Workflow

#### Frontend Development (Vue 3)
- Edit files in `src/` directory
- Changes hot-reload automatically
- Check browser console for errors (F12 in the app window)

#### Backend Development (Rust)
- Edit files in `src-tauri/src/` directory
- App rebuilds automatically on save
- Check terminal for compilation errors

### Project Structure

```
curl-master/
├── src/                         # Vue 3 frontend
│   ├── main.ts                 # App entry point
│   ├── App.vue                 # Root component
│   └── style.css               # Global styles
│
├── src-tauri/                   # Rust backend
│   ├── src/
│   │   ├── main.rs             # Tauri app entry
│   │   ├── http.rs             # HTTP executor (quad clients)
│   │   ├── db.rs               # Database operations
│   │   ├── models.rs           # Type-safe models
│   │   └── commands.rs         # Tauri commands (frontend ↔ backend)
│   ├── Cargo.toml              # Rust dependencies
│   └── tauri.conf.json         # Tauri configuration
│
├── documentation/
│   ├── implementation_plan.md  # Original architecture plan
│   └── BUILD_PROGRESS.md       # Current status & next steps
│
├── public/                      # Static assets
├── package.json                # Node dependencies
└── vite.config.ts              # Vite configuration
```

## Key Technologies

### Frontend
- **Vue 3**: Reactive UI framework
- **TypeScript**: Type-safe JavaScript
- **Pinia**: State management
- **Vite**: Build tool & dev server

### Backend
- **Tauri**: Desktop app framework
- **Rust**: High-performance backend
- **reqwest**: HTTP client library
- **sqlx**: Async SQLite driver
- **tokio**: Async runtime

## Calling Rust from Vue

```typescript
import { invoke } from '@tauri-apps/api/core';

// Send HTTP request
const result = await invoke('send_request', {
  config: {
    method: 'GET',
    url: 'https://api.example.com/data',
    headers: [],
    params: [],
    body: { type: 'None' },
    auth: null,
    timeout_ms: 30000,
    ignore_tls: false,
    follow_redirects: true,
  }
});

// Cancel request
await invoke('cancel_request', { requestId: 'some-uuid' });

// Get settings
const settings = await invoke('get_settings');

// Update setting
await invoke('update_setting', { 
  key: 'timeout_ms', 
  value: '60000' 
});
```

## Database

SQLite database is automatically created at:
- Windows: `%APPDATA%/com.curlmaster.app/curlmaster.db`
- macOS: `~/Library/Application Support/com.curlmaster.app/curlmaster.db`
- Linux: `~/.local/share/com.curlmaster.app/curlmaster.db`

Schema is in `src-tauri/src/db.rs` (auto-migrated on startup).

## Building for Production

```bash
npm run tauri build
```

Output will be in `src-tauri/target/release/bundle/`.

## Debugging

### Frontend Debugging
- Open DevTools in the app window (F12)
- Use Vue DevTools browser extension (if available)

### Backend Debugging
- Check terminal output for Rust logs
- Add `println!()` or `eprintln!()` for debugging
- Use `cargo check` for quick syntax checking:
  ```bash
  cd src-tauri
  cargo check
  ```

## Common Commands

```bash
# Install dependencies
npm install

# Run development server
npm run tauri dev

# Build for production
npm run tauri build

# Format Rust code
cd src-tauri && cargo fmt

# Check Rust code without building
cd src-tauri && cargo check

# Run Rust tests (when we add them)
cd src-tauri && cargo test
```

## Next Development Tasks

See `documentation/BUILD_PROGRESS.md` for detailed roadmap.

Priority order:
1. Custom titlebar component
2. Request builder panel
3. Response viewer panel
4. Collections sidebar
5. Tab system

## Notes

- The app uses a custom titlebar (no OS window decorations)
- All HTTP operations run in Rust for maximum performance
- Database operations are async via sqlx
- Type-safety is enforced throughout (Rust enums + TypeScript)

## Troubleshooting

### Port 5173 already in use
Kill the process using that port or change in `vite.config.ts`

### Rust compilation errors
- Ensure Rust is up to date: `rustup update`
- Clear cargo cache: `cd src-tauri && cargo clean`

### Database errors
Delete the database file to reset (will lose all data)

## Resources

- [Tauri Docs](https://tauri.app/v2/)
- [Vue 3 Docs](https://vuejs.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [reqwest Docs](https://docs.rs/reqwest/)
- [sqlx Docs](https://docs.rs/sqlx/)
