const Database = require('better-sqlite3');
const path = require('path');

const dbPath = process.env.SYNC_DB_PATH || path.join(__dirname, 'sync.db');
const db = new Database(dbPath);

db.exec(`
  CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL
  );

  CREATE TABLE IF NOT EXISTS workspace (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    owner_id TEXT NOT NULL REFERENCES users(id),
    updated_at INTEGER NOT NULL
  );

  CREATE TABLE IF NOT EXISTS workspace_member (
    user_id TEXT NOT NULL REFERENCES users(id),
    workspace_id TEXT NOT NULL REFERENCES workspace(id),
    role TEXT NOT NULL DEFAULT 'member',
    PRIMARY KEY (user_id, workspace_id)
  );

  CREATE TABLE IF NOT EXISTS workspace_invite (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspace(id),
    email TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE(workspace_id, email)
  );

  CREATE TABLE IF NOT EXISTS sync_snapshot (
    workspace_id TEXT PRIMARY KEY REFERENCES workspace(id),
    data TEXT NOT NULL,
    updated_at INTEGER NOT NULL
  );
`);

module.exports = { db };
