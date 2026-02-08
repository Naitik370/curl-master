# Curlmaster Sync Server

Backend for collaboration: auth, workspace membership, and workspace snapshot sync.

## Setup

```bash
cd sync-server
npm install
```

## Run

```bash
npm start
# or with env
PORT=3131 JWT_SECRET=your-secret npm start
```

- `PORT` – default 3131
- `JWT_SECRET` – secret for JWT (set in production)
- `SYNC_DB_PATH` – path to SQLite DB file (default: `./sync.db`)
- `APP_URL` – base URL for invite links (e.g. `https://sync.yourapp.com`)

## API

- `POST /auth/register` – `{ email, password }` → `{ id, email, token }`
- `POST /auth/login` – `{ email, password }` → `{ id, email, token }`
- `GET /workspaces` – Bearer token → list workspaces user is member of
- `PUT /sync/workspaces/:id` – Bearer token, body = full workspace snapshot (last-write-wins)
- `GET /sync/workspaces/:id` – Bearer token → full workspace snapshot
- `POST /workspaces/:id/invite` – `{ email }` → `{ inviteId, inviteLink }`
- `POST /invite/:inviteId/accept` – Bearer token → add user to workspace, return `{ workspaceId }`
