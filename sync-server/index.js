const express = require('express');
const cors = require('cors');
const { db } = require('./db');
const { register, login, authMiddleware } = require('./auth');

const app = express();
app.use(cors());
app.use(express.json({ limit: '10mb' }));

const PORT = process.env.PORT || 3131;

// --- Auth ---
app.post('/auth/register', (req, res) => {
  const { email, password } = req.body || {};
  if (!email || !password) {
    return res.status(400).json({ error: 'Email and password required' });
  }
  try {
    const result = register(email, password);
    res.json(result);
  } catch (e) {
    if (e.code === 'SQLITE_CONSTRAINT_UNIQUE') {
      return res.status(409).json({ error: 'Email already registered' });
    }
    throw e;
  }
});

app.post('/auth/login', (req, res) => {
  const { email, password } = req.body || {};
  if (!email || !password) {
    return res.status(400).json({ error: 'Email and password required' });
  }
  const result = login(email, password);
  if (!result) {
    return res.status(401).json({ error: 'Invalid email or password' });
  }
  res.json(result);
});

// --- Workspaces (list for current user) ---
app.get('/workspaces', authMiddleware, (req, res) => {
  const rows = db.prepare(`
    SELECT w.id, w.name, w.owner_id, w.updated_at
    FROM workspace w
    LEFT JOIN workspace_member m ON m.workspace_id = w.id
    WHERE w.owner_id = ? OR m.user_id = ?
    ORDER BY w.updated_at DESC
  `).all(req.userId, req.userId);
  res.json(rows);
});

// --- Invite (create invite; accept = add member when they have account) ---
app.post('/workspaces/:workspaceId/invite', authMiddleware, (req, res) => {
  const { workspaceId } = req.params;
  const { email } = req.body || {};
  if (!email) return res.status(400).json({ error: 'Email required' });

  const ws = db.prepare('SELECT id, owner_id FROM workspace WHERE id = ?').get(workspaceId);
  if (!ws) return res.status(404).json({ error: 'Workspace not found' });
  if (ws.owner_id !== req.userId) return res.status(403).json({ error: 'Only owner can invite' });

  const inviteId = require('crypto').randomUUID();
  try {
    db.prepare(
      'INSERT INTO workspace_invite (id, workspace_id, email, created_at) VALUES (?, ?, ?, ?)'
    ).run(inviteId, workspaceId, email.toLowerCase(), Date.now());
  } catch (e) {
    if (e.code === 'SQLITE_CONSTRAINT_UNIQUE') {
      return res.status(409).json({ error: 'Already invited' });
    }
    throw e;
  }
  const baseUrl = process.env.APP_URL || 'http://localhost:3131';
  res.json({ inviteId, inviteLink: `${baseUrl}/invite/${inviteId}` });
});

// Accept invite: add user as member (user must be logged in)
app.post('/invite/:inviteId/accept', authMiddleware, (req, res) => {
  const { inviteId } = req.params;
  const invite = db.prepare('SELECT workspace_id, email FROM workspace_invite WHERE id = ?').get(inviteId);
  if (!invite) return res.status(404).json({ error: 'Invite not found' });

  const user = db.prepare('SELECT id, email FROM users WHERE id = ?').get(req.userId);
  if (!user || user.email !== invite.email) {
    return res.status(403).json({ error: 'Invite was sent to a different email' });
  }

  db.prepare(
    'INSERT OR IGNORE INTO workspace_member (user_id, workspace_id, role) VALUES (?, ?, ?)'
  ).run(req.userId, invite.workspace_id, 'member');
  db.prepare('DELETE FROM workspace_invite WHERE id = ?').run(inviteId);
  res.json({ workspaceId: invite.workspace_id });
});

// --- Sync: PUT workspace snapshot ---
app.put('/sync/workspaces/:id', authMiddleware, (req, res) => {
  const workspaceId = req.params.id;
  const snapshot = req.body;

  if (!snapshot || typeof snapshot !== 'object') {
    return res.status(400).json({ error: 'Invalid snapshot' });
  }

  const ws = db.prepare('SELECT id, owner_id FROM workspace WHERE id = ?').get(workspaceId);
  const member = db.prepare('SELECT 1 FROM workspace_member WHERE workspace_id = ? AND user_id = ?').get(workspaceId, req.userId);

  if (!ws) {
    const now = Date.now();
    db.prepare('INSERT INTO workspace (id, name, owner_id, updated_at) VALUES (?, ?, ?, ?)').run(
      workspaceId,
      snapshot.workspace?.name || workspaceId,
      req.userId,
      now
    );
    db.prepare('INSERT OR IGNORE INTO workspace_member (user_id, workspace_id, role) VALUES (?, ?, ?)').run(req.userId, workspaceId, 'owner');
  } else if (ws.owner_id !== req.userId && !member) {
    return res.status(403).json({ error: 'Not a member of this workspace' });
  }

  const now = Date.now();
  const data = JSON.stringify(snapshot);
  db.prepare(
    'INSERT INTO sync_snapshot (workspace_id, data, updated_at) VALUES (?, ?, ?) ON CONFLICT(workspace_id) DO UPDATE SET data = excluded.data, updated_at = excluded.updated_at'
  ).run(workspaceId, data, now);

  db.prepare('UPDATE workspace SET name = ?, updated_at = ? WHERE id = ?').run(
    snapshot.workspace?.name ?? workspaceId,
    now,
    workspaceId
  );

  res.json({ updated_at: now });
});

// --- Sync: GET workspace snapshot ---
app.get('/sync/workspaces/:id', authMiddleware, (req, res) => {
  const workspaceId = req.params.id;
  const ws = db.prepare('SELECT id, owner_id FROM workspace WHERE id = ?').get(workspaceId);
  if (!ws) return res.status(404).json({ error: 'Workspace not found' });

  const member = db.prepare('SELECT 1 FROM workspace_member WHERE workspace_id = ? AND user_id = ?').get(workspaceId, req.userId);
  const isOwner = ws.owner_id === req.userId;
  if (!isOwner && !member) return res.status(403).json({ error: 'Not a member of this workspace' });

  const row = db.prepare('SELECT data, updated_at FROM sync_snapshot WHERE workspace_id = ?').get(workspaceId);
  if (!row) return res.status(404).json({ error: 'No snapshot yet' });

  const snapshot = JSON.parse(row.data);
  res.set('Last-Modified', new Date(row.updated_at).toUTCString());
  res.json({ ...snapshot, updated_at: row.updated_at });
});

app.listen(PORT, () => {
  console.log(`Curlmaster sync server listening on http://localhost:${PORT}`);
});
