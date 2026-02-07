# CurlMaster

A powerful, modern API client built with Tauri, Vue 3, and SQLite.

## Features

- 🚀 **Fast & Lightweight** - Built with Tauri for native performance
- 📦 **Collections & Folders** - Organize your API requests
- 🌍 **Environments** - Manage variables across different environments
- 🔒 **Security** - TLS options and authentication support
- 📝 **Request History** - Track all your API calls
- 💾 **Local Storage** - SQLite database for data persistence
- 🎨 **Modern UI** - Beautiful Vue 3 interface with dark mode

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Pinia + Vite
- **Backend**: Tauri 2.0 + Rust
- **HTTP**: reqwest with quad-client architecture
- **Database**: SQLite with sqlx

## Development

### Prerequisites

- Node.js 18+
- Rust 1.70+
- npm or yarn

### Install Dependencies

```bash
npm install
```

### Run Development Server

```bash
npm run tauri dev
```

### Build for Production

```bash
npm run tauri build
```

## Architecture Highlights

### Quad HTTP Clients

Four pre-configured reqwest clients for all TLS/redirect combinations:
- Strict + Follow Redirects
- Strict + No Redirects
- Insecure + Follow Redirects
- Insecure + No Redirects

### True Request Cancellation

Uses Rust futures with `AbortHandle` for guaranteed request cancellation.

### Type-Safe Models

Tagged enums prevent invalid states throughout the application.

## License

See LICENSE file for details.
