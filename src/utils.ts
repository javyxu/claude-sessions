import { existsSync } from 'node:fs';

// ── terminal colors ─────────────────────────────────────────────

const OSC = '\x1b';
export const bold = `${OSC}[1m`;
export const red = `${OSC}[31m`;
export const green = `${OSC}[32m`;
export const yellow = `${OSC}[33m`;
export const blue = `${OSC}[34m`;
export const magenta = `${OSC}[35m`;
export const cyan = `${OSC}[36m`;
export const reset = `${OSC}[0m`;

// ── formatting ──────────────────────────────────────────────────

/** Format bytes to human-readable string. */
export function humanSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1048576) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / 1048576).toFixed(1)} MB`;
}

/** Format a Unix millisecond timestamp to ISO-like string. */
export function formatTime(ms: number | string): string {
  const n = Number(ms);
  if (!n) return 'unknown';
  const d = new Date(n);
  if (isNaN(d.getTime())) return String(ms);
  return d.toISOString().replace('T', ' ').slice(0, 19);
}

// ── path decoding ───────────────────────────────────────────────

/**
 * Encode a filesystem path to the sanitized project directory name.
 *
 * Claude Code replaces "/" with "-" and prepends "-" for absolute paths:
 *   /home/user/my-app → -home-user-my-app
 */
export function encodeProject(cwd: string): string {
  const absolute = cwd.startsWith('/') ? cwd : '/' + cwd;
  return '-' + absolute.slice(1).replace(/\//g, '-');
}

/**
 * Decode a sanitized project directory name back to a real path.
 *
 * Claude Code replaces "/" with "-" and prepends "-" for absolute paths:
 *   /home/user/my-app → -home-user-my-app
 *
 * Hyphens in actual directory names are indistinguishable from encoded
 * path separators, so we validate against the filesystem.
 */
export function decodeProject(encoded: string): string {
  let decoded: string;
  if (encoded.startsWith('-')) {
    decoded = '/' + encoded.slice(1).replace(/-/g, '/');
  } else {
    decoded = encoded;
  }

  // Validate — if the decoded path exists, it's correct
  try {
    if (existsSync(decoded)) return decoded;
  } catch { /* fall through */ }

  // Try finding the longest valid prefix
  const parts = decoded.split('/');
  for (let i = parts.length; i >= 1; i--) {
    const candidate = parts.slice(0, i).join('/') || '/';
    try {
      if (existsSync(candidate)) return decoded + ' [?]';
    } catch { /* continue */ }
  }

  return decoded + ' [?]';
}

// ── safe JSON ───────────────────────────────────────────────────

export function safeJson<T = Record<string, unknown>>(
  str: string,
  fallback: T = {} as T,
): T {
  try {
    return JSON.parse(str) as T;
  } catch {
    return fallback;
  }
}
