import {
  existsSync,
  readFileSync,
  readdirSync,
  statSync,
  lstatSync,
  unlinkSync,
  rmSync,
} from 'node:fs';
import { join, basename } from 'node:path';
import { homedir } from 'node:os';
import { createInterface } from 'node:readline';
import {
  bold, red, green, yellow, blue, magenta, cyan, reset,
  humanSize, decodeProject, encodeProject, formatTime, safeJson,
} from './utils.js';

// ── paths ───────────────────────────────────────────────────────

const CLAUDE_DIR = join(homedir(), '.claude');
const PROJECTS_DIR = join(CLAUDE_DIR, 'projects');
const SESSIONS_DIR = join(CLAUDE_DIR, 'sessions');
const SESSION_ENV_DIR = join(CLAUDE_DIR, 'session-env');
const FILE_HISTORY_DIR = join(CLAUDE_DIR, 'file-history');

// ── types ───────────────────────────────────────────────────────

interface SessionMeta {
  sessionId: string;
  projectEncoded: string;
  projectDecoded: string;
  status: 'active' | 'inactive';
  lineCount: number;
  fileSize: number;
  mtime: number;
  leafUuid: string;
  filePath: string;
  name?: string;
}

interface ActiveSession {
  pid?: number;
  sessionId?: string;
  cwd?: string;
  status?: string;
  updatedAt?: number;
}

interface SessionHeader {
  leafUuid?: string;
  type?: string;
}

// ── helpers ─────────────────────────────────────────────────────

function readFirstLine(filePath: string): string {
  try {
    const content = readFileSync(filePath, 'utf8');
    const nl = content.indexOf('\n');
    return nl >= 0 ? content.slice(0, nl) : content;
  } catch {
    return '{}';
  }
}

function getActiveSessions(): Set<string> {
  const ids = new Set<string>();
  try {
    for (const file of readdirSync(SESSIONS_DIR)) {
      if (!file.endsWith('.json')) continue;
      try {
        const data = safeJson<{ sessionId?: string }>(
          readFileSync(join(SESSIONS_DIR, file), 'utf8'),
        );
        if (data.sessionId) ids.add(data.sessionId);
      } catch { /* skip */ }
    }
  } catch { /* dir may not exist */ }
  return ids;
}

function getActiveSessionMeta(): ActiveSession[] {
  const result: ActiveSession[] = [];
  try {
    for (const file of readdirSync(SESSIONS_DIR)) {
      if (!file.endsWith('.json')) continue;
      try {
        result.push(
          safeJson<ActiveSession>(readFileSync(join(SESSIONS_DIR, file), 'utf8')),
        );
      } catch { /* skip */ }
    }
  } catch { /* dir may not exist */ }
  return result;
}

function gatherSessions(opts: { projectFilter?: string } = {}): SessionMeta[] {
  const activeIds = getActiveSessions();
  // Build sessionId → name map from active session metadata
  const nameMap = new Map<string, string>();
  for (const s of getActiveSessionMeta()) {
    if (s.sessionId && s.name) nameMap.set(s.sessionId, s.name);
  }
  const results: SessionMeta[] = [];

  let projectDirs: string[] = [];
  try {
    projectDirs = readdirSync(PROJECTS_DIR, { withFileTypes: true })
      .filter(d => d.isDirectory())
      .map(d => d.name);
  } catch { return results; }

  for (const encoded of projectDirs) {
    const decoded = decodeProject(encoded);

    // Fuzzy project filter
    if (opts.projectFilter) {
      const lower = opts.projectFilter.toLowerCase();
      if (
        !encoded.toLowerCase().includes(lower) &&
        !decoded.toLowerCase().includes(lower)
      ) {
        continue;
      }
    }

    const projectDir = join(PROJECTS_DIR, encoded);
    let files: string[] = [];
    try {
      files = readdirSync(projectDir).filter(f => f.endsWith('.jsonl'));
    } catch { continue; }

    for (const file of files) {
      const sessionId = file.replace(/\.jsonl$/, '');
      const fullPath = join(projectDir, file);

      let size = 0;
      let mtime = 0;
      try {
        const st = statSync(fullPath);
        size = st.size;
        mtime = st.mtimeMs;
      } catch { /* keep defaults */ }

      let lineCount = 0;
      try {
        const content = readFileSync(fullPath, 'utf8');
        lineCount = (content.match(/\n/g) || []).length;
      } catch { /* keep 0 */ }

      const firstLine = readFirstLine(fullPath);
      const header = safeJson<SessionHeader>(firstLine);
      const leafUuid = header.leafUuid || '';

      results.push({
        sessionId,
        projectEncoded: encoded,
        projectDecoded: decoded,
        status: activeIds.has(sessionId) ? 'active' : 'inactive',
        lineCount,
        fileSize: size,
        mtime,
        leafUuid,
        filePath: fullPath,
        name: nameMap.get(sessionId),
      });
    }
  }

  results.sort((a, b) => b.mtime - a.mtime);
  return results;
}

// ── commands ─────────────────────────────────────────────────────

export function list(opts: {
  project?: string;
  limit?: number;
  json?: boolean;
  all?: boolean;
} = {}): void {
  // Default to current project unless --all or explicit --project
  const filter = opts.all ? undefined : (opts.project || encodeProject(process.cwd()));
  const sessions = gatherSessions({ projectFilter: filter });
  const activeMeta = getActiveSessionMeta();
  const activeCount = activeMeta.length;

  if (opts.json) {
    const limited = opts.limit ? sessions.slice(0, opts.limit) : sessions;
    for (const s of limited) {
      process.stdout.write(JSON.stringify({
        sessionId: s.sessionId,
        project: s.projectDecoded,
        status: s.status,
        lineCount: s.lineCount,
        fileSize: s.fileSize,
        mtime: s.mtime,
        lastPromptLeaf: s.leafUuid,
      }) + '\n');
    }
    return;
  }

  const displayed = opts.limit ? sessions.slice(0, opts.limit) : sessions;
  const sep = '─'.repeat(30);

  console.log(`${bold}${blue}=== Claude Code Sessions ===${reset}`);
  console.log(
    `${cyan}${'NAME'.padEnd(16)}  ${'SESSION ID'.padEnd(38)}  ${'STATUS'.padEnd(10)}  ` +
    `${'LINES'.padEnd(6)}  ${'SIZE'.padEnd(7)}  PROJECT${reset}`,
  );
  console.log(
    `${'─'.repeat(16)}  ${'─'.repeat(38)}  ${'─'.repeat(10)}  ${'─'.repeat(6)}  ` +
    `${'─'.repeat(7)}  ${sep}`,
  );

  for (const s of displayed) {
    const icon = s.status === 'active'
      ? `${green}● active  ${reset}`
      : `${yellow}○ idle    ${reset}`;

    const name = s.name || '—';

    console.log(
      `${name.slice(0, 16).padEnd(16)}  ` +
      `${bold}${s.sessionId}${reset}  ` +
      `${icon}  ` +
      `${String(s.lineCount).padEnd(6)}  ` +
      `${humanSize(s.fileSize).padEnd(7)}  ` +
      `${magenta}${s.projectDecoded}${reset}`,
    );
  }

  if (sessions.length === 0) {
    console.log(`${yellow}No sessions found.${reset}`);
  } else {
    console.log(
      `\n${bold}${sessions.length} session(s) total${reset}, ` +
      `${green}${activeCount} active${reset}`,
    );
  }
}

export function show(sessionId: string): void {
  let foundPath = '';
  let projectEncoded = '';

  try {
    for (const d of readdirSync(PROJECTS_DIR, { withFileTypes: true })) {
      if (!d.isDirectory()) continue;
      const jsonlPath = join(PROJECTS_DIR, d.name, `${sessionId}.jsonl`);
      if (existsSync(jsonlPath)) {
        foundPath = jsonlPath;
        projectEncoded = d.name;
        break;
      }
    }
  } catch { /* ignore */ }

  if (!foundPath) {
    console.error(`${red}Session '${sessionId}' not found.${reset}`);
    process.exit(1);
  }

  const projectDecoded = decodeProject(projectEncoded);
  const stat = statSync(foundPath);
  const content = readFileSync(foundPath, 'utf8');
  const lineCount = (content.match(/\n/g) || []).length;
  const firstLine = content.split('\n')[0] || '{}';
  const firstJson = safeJson<SessionHeader>(firstLine);

  let mode = 'unknown';
  let permissionMode = 'unknown';
  for (const line of content.split('\n')) {
    if (!line.trim()) continue;
    try {
      const entry = JSON.parse(line) as Record<string, unknown>;
      if (entry.type === 'mode' && entry.mode) mode = String(entry.mode);
      if (entry.type === 'permission-mode' && entry.permissionMode) {
        permissionMode = String(entry.permissionMode);
      }
    } catch { /* skip */ }
  }

  const activeIds = getActiveSessions();
  const isActive = activeIds.has(sessionId);

  console.log(`${bold}${blue}=== Session: ${sessionId} ===${reset}`);
  console.log(`Project:       ${magenta}${projectDecoded}${reset}`);
  console.log(
    `Status:        ${isActive ? green + 'active' + reset : 'inactive'}`,
  );
  console.log(`Mode:          ${mode}`);
  console.log(`Permission:    ${permissionMode}`);
  console.log(
    `Transcript:    ${lineCount} lines (${humanSize(stat.size)})`,
  );
  console.log(`Leaf UUID:     ${firstJson.leafUuid || 'N/A'}`);
  console.log(`File:          ${foundPath}`);

  const envPath = join(SESSION_ENV_DIR, sessionId);
  if (existsSync(envPath)) {
    console.log(`Session Env:   ${envPath}`);
  }

  try {
    const historyFiles = readdirSync(FILE_HISTORY_DIR);
    const matching = historyFiles.filter(f => f.startsWith(sessionId));
    if (matching.length > 0) {
      console.log(
        `File History:  ${join(FILE_HISTORY_DIR, sessionId)}* ` +
        `(${matching.length} entries)`,
      );
    }
  } catch { /* ignore */ }
}

export function remove(sessionId: string, opts: { force?: boolean } = {}): void {
  let foundPath = '';
  let projectEncoded = '';

  try {
    for (const d of readdirSync(PROJECTS_DIR, { withFileTypes: true })) {
      if (!d.isDirectory()) continue;
      const jsonlPath = join(PROJECTS_DIR, d.name, `${sessionId}.jsonl`);
      if (existsSync(jsonlPath)) {
        foundPath = jsonlPath;
        projectEncoded = d.name;
        break;
      }
    }
  } catch { /* ignore */ }

  if (!foundPath) {
    console.error(`${red}Session '${sessionId}' not found.${reset}`);
    process.exit(1);
  }

  const toDelete: string[] = [foundPath];

  // session-env
  const envPath = join(SESSION_ENV_DIR, sessionId);
  if (existsSync(envPath)) toDelete.push(envPath);

  // file-history
  try {
    for (const f of readdirSync(FILE_HISTORY_DIR)) {
      if (f.startsWith(sessionId)) {
        toDelete.push(join(FILE_HISTORY_DIR, f));
      }
    }
  } catch { /* ignore */ }

  // Check active
  let activePidFile = '';
  try {
    for (const f of readdirSync(SESSIONS_DIR)) {
      if (!f.endsWith('.json')) continue;
      const fp = join(SESSIONS_DIR, f);
      const data = safeJson<{ sessionId?: string }>(readFileSync(fp, 'utf8'));
      if (data.sessionId === sessionId) {
        activePidFile = fp;
        break;
      }
    }
  } catch { /* ignore */ }

  if (activePidFile && !opts.force) {
    console.error(
      `${yellow}⚠ Session '${sessionId}' is currently active!${reset}`,
    );
    console.error(
      'Use --force to delete an active session, or end the session first.',
    );
    process.exit(1);
  }
  if (activePidFile) toDelete.push(activePidFile);

  // Preview
  const projectDecoded = decodeProject(projectEncoded);
  console.log(`${bold}=== Preparing to delete session ===${reset}`);
  console.log(`Session:  ${bold}${sessionId}${reset}`);
  console.log(`Project:  ${magenta}${projectDecoded}${reset}`);
  console.log('\nFiles to remove:');
  for (const f of toDelete) {
    console.log(`  ${red}✗${reset} ${f}`);
  }

  // Confirm (non-interactive when force)
  if (!opts.force) {
    prompt(`\n${yellow}Confirm deletion? (y/N)${reset}: `).then(answer => {
      if (answer.toLowerCase() !== 'y') {
        console.log('Aborted.');
        return;
      }
      doDelete(toDelete, sessionId);
    });
    return;
  }

  doDelete(toDelete, sessionId);
}

function doDelete(files: string[], sessionId: string): void {
  for (const f of files) {
    try {
      const st = lstatSync(f);
      if (st.isDirectory()) {
        rmSync(f, { recursive: true, force: true });
      } else {
        unlinkSync(f);
      }
      console.log(`${green}✓ Deleted:${reset} ${f}`);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      console.error(`${red}✗ Failed:${reset} ${f} (${msg})`);
    }
  }
  console.log(
    `\n${bold}${green}Session '${sessionId}' deleted successfully.${reset}`,
  );
}

export function projects(): void {
  const activeIds = getActiveSessions();

  console.log(`${bold}${blue}=== Projects with Sessions ===${reset}\n`);

  let projectDirs: string[] = [];
  try {
    projectDirs = readdirSync(PROJECTS_DIR, { withFileTypes: true })
      .filter(d => d.isDirectory())
      .map(d => d.name);
  } catch {
    console.log(`${yellow}No projects found.${reset}`);
    return;
  }

  let grandTotal = 0;
  for (const dirName of projectDirs) {
    const projectDir = join(PROJECTS_DIR, dirName);
    const files = readdirSync(projectDir).filter(f => f.endsWith('.jsonl'));
    const sessionCount = files.length;
    let activeCount = 0;

    for (const f of files) {
      const sid = f.replace(/\.jsonl$/, '');
      if (activeIds.has(sid)) activeCount++;
    }

    const decoded = decodeProject(dirName);
    console.log(
      `${magenta}${decoded}${reset} ` +
      `${cyan}(${sessionCount} sessions, ${activeCount} active)${reset}`,
    );
    grandTotal += sessionCount;
  }

  console.log(
    `\n${bold}${grandTotal} total${reset} sessions ` +
    `across ${projectDirs.length} project(s)`,
  );
}

export function active(): void {
  const sessions = getActiveSessionMeta();

  console.log(`${bold}${blue}=== Active Sessions ===${reset}\n`);

  if (sessions.length === 0) {
    console.log(`${yellow}No active sessions.${reset}`);
    return;
  }

  for (const s of sessions) {
    const updatedFmt = formatTime(s.updatedAt ?? 0);
    console.log(
      `PID: ${String(s.pid ?? '?').padEnd(8)}  ` +
      `Status: ${green}${String(s.status ?? '?').padEnd(8)}${reset}  ` +
      `CWD: ${s.cwd ?? '?'}`,
    );
    console.log(
      `  Session: ${bold}${s.sessionId ?? '?'}${reset}  ` +
      `Updated: ${updatedFmt}`,
    );
  }
}

// ── interactive prompt ───────────────────────────────────────────

function prompt(question: string): Promise<string> {
  return new Promise(resolve => {
    process.stdout.write(question);
    const rl = createInterface({ input: process.stdin, output: process.stdout });
    rl.on('line', line => {
      rl.close();
      resolve(line.trim());
    });
  });
}
