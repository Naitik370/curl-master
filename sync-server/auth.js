const bcrypt = require('bcryptjs');
const jwt = require('jsonwebtoken');
const { db } = require('./db');
const crypto = require('crypto');

const JWT_SECRET = process.env.JWT_SECRET || 'curlmaster-sync-secret-change-in-production';
const SALT_ROUNDS = 10;

function hashPassword(password) {
  return bcrypt.hashSync(password, SALT_ROUNDS);
}

function verifyPassword(password, hash) {
  return bcrypt.compareSync(password, hash);
}

function createToken(userId) {
  return jwt.sign({ userId }, JWT_SECRET, { expiresIn: '7d' });
}

function verifyToken(token) {
  try {
    const decoded = jwt.verify(token, JWT_SECRET);
    return decoded.userId;
  } catch {
    return null;
  }
}

function register(email, password) {
  const id = crypto.randomUUID();
  const password_hash = hashPassword(password);
  const created_at = Date.now();
  db.prepare(
    'INSERT INTO users (id, email, password_hash, created_at) VALUES (?, ?, ?, ?)'
  ).run(id, email.toLowerCase(), password_hash, created_at);
  return { id, email: email.toLowerCase(), token: createToken(id) };
}

function login(email, password) {
  const row = db.prepare('SELECT id, email, password_hash FROM users WHERE email = ?').get(email.toLowerCase());
  if (!row || !verifyPassword(password, row.password_hash)) {
    return null;
  }
  return { id: row.id, email: row.email, token: createToken(row.id) };
}

function authMiddleware(req, res, next) {
  const authHeader = req.headers.authorization;
  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return res.status(401).json({ error: 'Missing or invalid authorization' });
  }
  const token = authHeader.slice(7);
  const userId = verifyToken(token);
  if (!userId) {
    return res.status(401).json({ error: 'Invalid or expired token' });
  }
  req.userId = userId;
  next();
}

module.exports = {
  register,
  login,
  authMiddleware,
  createToken,
  verifyToken,
};
