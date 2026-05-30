use crate::utils::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

// ── paths ───────────────────────────────────────────────────────

fn claude_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    PathBuf::from(home).join(".claude")
}

fn projects_dir() -> PathBuf {
    claude_dir().join("projects")
}

fn sessions_dir() -> PathBuf {
    claude_dir().join("sessions")
}

fn session_env_dir() -> PathBuf {
    claude_dir().join("session-env")
}

fn file_history_dir() -> PathBuf {
    claude_dir().join("file-history")
}

// ── types ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SessionMeta {
    pub session_id: String,
    pub project_encoded: String,
    pub project_decoded: String,
    pub status: String, // "active" | "inactive"
    pub line_count: usize,
    pub file_size: u64,
    pub mtime: i64,
    pub leaf_uuid: String,
    pub file_path: PathBuf,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ActiveSession {
    pid: Option<u32>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    cwd: Option<String>,
    status: Option<String>,
    #[serde(rename = "updatedAt")]
    updated_at: Option<i64>,
    name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct SessionHeader {
    #[serde(rename = "leafUuid")]
    leaf_uuid: Option<String>,
}

// ── helpers ─────────────────────────────────────────────────────

fn read_first_line(file_path: &std::path::Path) -> String {
    if let Ok(file) = fs::File::open(file_path) {
        let reader = io::BufReader::new(file);
        if let Some(Ok(line)) = reader.lines().next() {
            return line;
        }
    }
    "{}".to_string()
}

/// Strip HTML/XML tags from a string, keeping inner text.
fn strip_tags(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result.trim().to_string()
}

/// Truncate a string to `max` width, appending "…" if cut.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

/// Read session name from the .jsonl transcript file.
/// Priority: agent-name > custom-title > ai-title > first user prompt.
/// Returns the chronologically last match within each priority tier.
fn read_session_name(file_path: &std::path::Path) -> Option<String> {
    let file = fs::File::open(file_path).ok()?;
    let reader = io::BufReader::new(file);
    let mut name = None;
    let mut first_prompt = None;
    for line in reader.lines().take(200) {
        let line = line.ok()?;
        if line.is_empty() {
            continue;
        }
        let val: serde_json::Value = serde_json::from_str(&line).ok()?;
        let ty = val.get("type")?.as_str()?;
        match ty {
            "agent-name" => {
                name = val.get("agentName")?.as_str().map(String::from);
            }
            "custom-title" => {
                // Only use custom-title if no agent-name yet
                if name.is_none() {
                    name = val.get("customTitle")?.as_str().map(String::from);
                }
            }
            "ai-title" => {
                // Only use ai-title if no agent-name or custom-title yet
                if name.is_none() {
                    name = val.get("aiTitle")?.as_str().map(String::from);
                }
            }
            "user" => {
                if first_prompt.is_none()
                    && val.get("isMeta") != Some(&serde_json::Value::Bool(true))
                {
                    first_prompt = val
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_str())
                        .map(|s| s.lines().next().unwrap_or(s))
                        .map(strip_tags)
                        .filter(|s| !s.is_empty());
                }
            }
            _ => continue,
        }
    }
    name.or(first_prompt)
}

fn get_active_session_ids() -> HashSet<String> {
    let mut ids = HashSet::new();
    let dir = sessions_dir();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(true, |e| e != "json") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(sid) = parsed["sessionId"].as_str() {
                        ids.insert(sid.to_string());
                    }
                }
            }
        }
    }
    ids
}

fn get_active_session_meta() -> Vec<ActiveSession> {
    let mut result = Vec::new();
    let dir = sessions_dir();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(true, |e| e != "json") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(session) = serde_json::from_str::<ActiveSession>(&content) {
                    result.push(session);
                }
            }
        }
    }
    result
}

pub fn gather_sessions(project_filter: Option<&str>) -> Vec<SessionMeta> {
    let active_ids = get_active_session_ids();
    let name_map: std::collections::HashMap<String, String> = get_active_session_meta()
        .into_iter()
        .filter_map(|s| Some((s.session_id?, s.name?)))
        .collect();

    let mut results = Vec::new();

    let dir = projects_dir();
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return results,
    };

    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let encoded = entry.file_name().to_string_lossy().to_string();
        let decoded = decode_project(&encoded);

        if let Some(filter) = project_filter {
            let lower = filter.to_lowercase();
            if !encoded.to_lowercase().contains(&lower)
                && !decoded.to_lowercase().contains(&lower)
            {
                continue;
            }
        }

        let project_dir = entry.path();
        let dir_entries = match fs::read_dir(&project_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for file_entry in dir_entries.flatten() {
            let path = file_entry.path();
            if path.extension().map_or(true, |e| e != "jsonl") {
                continue;
            }

            let session_id = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let (size, mtime) = match fs::metadata(&path) {
                Ok(m) => (m.len(), m.modified().ok()),
                Err(_) => (0, None),
            };

            let mtime_ms = mtime
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);

            let line_count = match fs::read_to_string(&path) {
                Ok(content) => content.lines().count(),
                Err(_) => 0,
            };

            let first_line = read_first_line(&path);
            let header: SessionHeader = safe_json(&first_line);
            let leaf_uuid = header.leaf_uuid.unwrap_or_default();

            results.push(SessionMeta {
                session_id: session_id.clone(),
                project_encoded: encoded.clone(),
                project_decoded: decoded.clone(),
                status: if active_ids.contains(&session_id) {
                    "active".to_string()
                } else {
                    "inactive".to_string()
                },
                line_count,
                file_size: size,
                mtime: mtime_ms,
                leaf_uuid,
                file_path: path.clone(),
                name: name_map
                    .get(&session_id)
                    .cloned()
                    .or_else(|| read_session_name(&path)),
            });
        }
    }

    results.sort_by(|a, b| b.mtime.cmp(&a.mtime));
    results
}

// ── commands ─────────────────────────────────────────────────────

pub fn list_sessions(project: Option<&str>, limit: Option<usize>, json: bool, all: bool) {
    let filter = if all {
        None
    } else {
        Some(project.map(String::from).unwrap_or_else(|| {
            encode_project(&std::env::current_dir().unwrap_or_default().to_string_lossy())
        }))
    };
    let filter_ref = filter.as_deref();

    let sessions = gather_sessions(filter_ref);
    let active_count = get_active_session_meta().len();

    if json {
        let iter: Box<dyn Iterator<Item = &SessionMeta>> = if let Some(n) = limit {
            Box::new(sessions.iter().take(n))
        } else {
            Box::new(sessions.iter())
        };
        for s in iter {
            let obj = serde_json::json!({
                "sessionId": s.session_id,
                "name": s.name,
                "project": s.project_decoded,
                "status": s.status,
                "lineCount": s.line_count,
                "fileSize": s.file_size,
                "mtime": s.mtime,
                "lastPromptLeaf": s.leaf_uuid,
            });
            println!("{}", serde_json::to_string(&obj).unwrap());
        }
        return;
    }

    let displayed: Vec<&SessionMeta> = if let Some(n) = limit {
        sessions.iter().take(n).collect()
    } else {
        sessions.iter().collect()
    };

    println!(
        "{BOLD}{BLUE}=== Claude Code Sessions ==={RESET}"
    );
    println!(
        "{CYAN}{:48}  {:38}  {:10}  {:6}  {:7}  PROJECT{RESET}",
        "NAME", "SESSION ID", "STATUS", "LINES", "SIZE"
    );
    println!(
        "{:─<48}  {:─<38}  {:─<10}  {:─<6}  {:─<7}  {:─<30}",
        "", "", "", "", "", ""
    );

    for s in displayed {
        let (icon, status_text) = if s.status == "active" {
            (format!("{GREEN}●{RESET}"), "active")
        } else {
            (format!("{YELLOW}○{RESET}"), "idle")
        };

        let name = truncate(s.name.as_deref().unwrap_or("—"), 48);
        let id_display = format!("{BOLD}{}{RESET}", s.session_id);

        println!(
            "{:48}  {:38}  {} {:7}  {:6}  {:7}  {MAGENTA}{}{RESET}",
            name,
            id_display,
            icon,
            status_text,
            s.line_count,
            human_size(s.file_size),
            s.project_decoded
        );
    }

    if sessions.is_empty() {
        println!("{YELLOW}No sessions found.{RESET}");
    } else {
        println!(
            "\n{BOLD}{} session(s) total{RESET}, {GREEN}{} active{RESET}",
            sessions.len(),
            active_count
        );
    }
}

pub fn show_session(session_id: &str) {
    let mut found_path = PathBuf::new();
    let mut project_encoded = String::new();

    if let Ok(entries) = fs::read_dir(projects_dir()) {
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let jsonl_path = entry.path().join(format!("{}.jsonl", session_id));
            if jsonl_path.exists() {
                found_path = jsonl_path;
                project_encoded = entry.file_name().to_string_lossy().to_string();
                break;
            }
        }
    }

    if found_path.as_os_str().is_empty() {
        eprintln!("{RED}Session '{}' not found.{RESET}", session_id);
        std::process::exit(1);
    }

    let project_decoded = decode_project(&project_encoded);
    let metadata = fs::metadata(&found_path).unwrap_or_else(|_| {
        eprintln!("{RED}Failed to read session file.{RESET}");
        std::process::exit(1);
    });
    let content = fs::read_to_string(&found_path).unwrap_or_default();
    let line_count = content.lines().count();
    let first_line = content.lines().next().unwrap_or("{}");
    let first_json: SessionHeader = safe_json(first_line);

    let mut mode = "unknown".to_string();
    let mut permission_mode = "unknown".to_string();
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            if entry.get("type").and_then(|v| v.as_str()) == Some("mode") {
                if let Some(m) = entry.get("mode").and_then(|v| v.as_str()) {
                    mode = m.to_string();
                }
            }
            if entry.get("type").and_then(|v| v.as_str()) == Some("permission-mode") {
                if let Some(pm) = entry.get("permissionMode").and_then(|v| v.as_str()) {
                    permission_mode = pm.to_string();
                }
            }
        }
    }

    let active_ids = get_active_session_ids();
    let is_active = active_ids.contains(session_id);

    println!("{BOLD}{BLUE}=== Session: {} ==={RESET}", session_id);
    println!("Project:       {MAGENTA}{}{RESET}", project_decoded);
    println!(
        "Status:        {}",
        if is_active {
            format!("{GREEN}active{RESET}")
        } else {
            "inactive".to_string()
        }
    );
    println!("Mode:          {}", mode);
    println!("Permission:    {}", permission_mode);
    println!(
        "Transcript:    {} lines ({})",
        line_count,
        human_size(metadata.len())
    );
    println!(
        "Leaf UUID:     {}",
        first_json.leaf_uuid.as_deref().unwrap_or("N/A")
    );
    println!("File:          {}", found_path.display());

    let env_path = session_env_dir().join(session_id);
    if env_path.exists() {
        println!("Session Env:   {}", env_path.display());
    }

    if let Ok(history_entries) = fs::read_dir(file_history_dir()) {
        let matching: Vec<String> = history_entries
            .flatten()
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with(session_id)
            })
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        if !matching.is_empty() {
            println!(
                "File History:  {}* ({} entries)",
                file_history_dir().join(session_id).display(),
                matching.len()
            );
        }
    }
}

pub fn remove_session(session_id: &str, force: bool) {
    let mut found_path = PathBuf::new();
    let mut project_encoded = String::new();

    if let Ok(entries) = fs::read_dir(projects_dir()) {
        for entry in entries.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let jsonl_path = entry.path().join(format!("{}.jsonl", session_id));
            if jsonl_path.exists() {
                found_path = jsonl_path;
                project_encoded = entry.file_name().to_string_lossy().to_string();
                break;
            }
        }
    }

    if found_path.as_os_str().is_empty() {
        eprintln!("{RED}Session '{}' not found.{RESET}", session_id);
        std::process::exit(1);
    }

    let mut to_delete: Vec<PathBuf> = vec![found_path.clone()];

    // session-env
    let env_path = session_env_dir().join(session_id);
    if env_path.exists() {
        to_delete.push(env_path);
    }

    // file-history
    if let Ok(history_entries) = fs::read_dir(file_history_dir()) {
        for entry in history_entries.flatten() {
            let name = entry.file_name();
            if name.to_string_lossy().starts_with(session_id) {
                to_delete.push(entry.path());
            }
        }
    }

    // Check active
    let mut active_pid_file = None;
    if let Ok(session_entries) = fs::read_dir(sessions_dir()) {
        for entry in session_entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(true, |e| e != "json") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if val["sessionId"].as_str() == Some(session_id) {
                        active_pid_file = Some(path);
                        break;
                    }
                }
            }
        }
    }

    if let Some(ref pid_file) = active_pid_file {
        if !force {
            eprintln!(
                "{YELLOW}⚠ Session '{}' is currently active!{RESET}",
                session_id
            );
            eprintln!("Use --force to delete an active session, or end the session first.");
            std::process::exit(1);
        }
        to_delete.push(pid_file.clone());
    }

    // Preview
    let project_decoded = decode_project(&project_encoded);
    println!("{BOLD}=== Preparing to delete session ==={RESET}");
    println!("Session:  {BOLD}{}{RESET}", session_id);
    println!("Project:  {MAGENTA}{}{RESET}", project_decoded);
    println!("\nFiles to remove:");
    for f in &to_delete {
        println!("  {RED}✗{RESET} {}", f.display());
    }

    if !force {
        let answer = prompt(&format!(
            "\n{YELLOW}Confirm deletion? (y/N){RESET}: "
        ));
        if answer.to_lowercase() != "y" {
            println!("Aborted.");
            return;
        }
    }

    do_delete(&to_delete, session_id);
}

fn do_delete(files: &[PathBuf], session_id: &str) {
    for f in files {
        let result = if f.is_dir() {
            fs::remove_dir_all(f)
        } else {
            fs::remove_file(f)
        };
        match result {
            Ok(()) => println!("{GREEN}✓ Deleted:{RESET} {}", f.display()),
            Err(e) => eprintln!("{RED}✗ Failed:{RESET} {} ({})", f.display(), e),
        }
    }
    println!(
        "\n{BOLD}{GREEN}Session '{}' deleted successfully.{RESET}",
        session_id
    );
}

pub fn clear_sessions(project: Option<&str>, force: bool) {
    let sessions = gather_sessions(project);

    if sessions.is_empty() {
        println!("{YELLOW}No sessions found to clear.{RESET}");
        return;
    }

    // Collect all files to delete
    let mut to_delete: Vec<PathBuf> = Vec::new();
    for s in &sessions {
        to_delete.push(s.file_path.clone());
        let env_path = session_env_dir().join(&s.session_id);
        if env_path.exists() {
            to_delete.push(env_path);
        }
        if let Ok(history_entries) = fs::read_dir(file_history_dir()) {
            for entry in history_entries.flatten() {
                if entry.file_name().to_string_lossy().starts_with(&s.session_id) {
                    to_delete.push(entry.path());
                }
            }
        }
        if let Ok(session_entries) = fs::read_dir(sessions_dir()) {
            for entry in session_entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(true, |e| e != "json") {
                    continue;
                }
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                        if val["sessionId"].as_str() == Some(&s.session_id) {
                            to_delete.push(path);
                        }
                    }
                }
            }
        }
    }

    // Preview
    let scope = if let Some(p) = project {
        decode_project(p)
    } else {
        "ALL projects".to_string()
    };
    println!("{BOLD}{RED}=== Preparing to clear sessions ==={RESET}");
    println!("Scope:    {BOLD}{}{RESET}", scope);
    println!("Sessions: {BOLD}{}{RESET}", sessions.len());
    println!("Files:    {BOLD}{}{RESET}", to_delete.len());

    if !force {
        let answer = prompt(&format!(
            "\n{RED}Delete {} sessions and {} files? (y/N){RESET}: ",
            sessions.len(),
            to_delete.len()
        ));
        if answer.to_lowercase() != "y" {
            println!("Aborted.");
            return;
        }
    }

    let mut ok = 0usize;
    let mut fail = 0usize;
    for f in &to_delete {
        let result = if f.is_dir() {
            fs::remove_dir_all(f)
        } else {
            fs::remove_file(f)
        };
        match result {
            Ok(()) => ok += 1,
            Err(e) => {
                fail += 1;
                eprintln!("{RED}✗ Failed:{RESET} {} ({})", f.display(), e);
            }
        }
    }

    println!(
        "\n{BOLD}{GREEN}Cleared {ok} files ({fail} failed){RESET}"
    );
}

pub fn list_projects() {
    let active_ids = get_active_session_ids();

    println!("{BOLD}{BLUE}=== Projects with Sessions ==={RESET}\n");

    let entries = match fs::read_dir(projects_dir()) {
        Ok(e) => e,
        Err(_) => {
            println!("{YELLOW}No projects found.{RESET}");
            return;
        }
    };

    let mut project_dirs: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            project_dirs.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    let mut grand_total = 0usize;
    for dir_name in &project_dirs {
        let project_dir = projects_dir().join(dir_name);
        let files: Vec<_> = fs::read_dir(&project_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "jsonl")
            })
            .collect();

        let session_count = files.len();
        let mut active_count = 0usize;
        for f in &files {
            let path = f.path();
            let sid = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            if active_ids.contains(sid.as_ref()) {
                active_count += 1;
            }
        }

        let decoded = decode_project(dir_name);
        println!(
            "{MAGENTA}{}{RESET} {CYAN}({} sessions, {} active){RESET}",
            decoded, session_count, active_count
        );
        grand_total += session_count;
    }

    println!(
        "\n{BOLD}{} total{RESET} sessions across {} project(s)",
        grand_total,
        project_dirs.len()
    );
}

pub fn list_active() {
    let sessions = get_active_session_meta();

    println!("{BOLD}{BLUE}=== Active Sessions ==={RESET}\n");

    if sessions.is_empty() {
        println!("{YELLOW}No active sessions.{RESET}");
        return;
    }

    for s in &sessions {
        let pid = s.pid.map(|p| p.to_string()).unwrap_or_else(|| "?".to_string());
        let status = s.status.as_deref().unwrap_or("?");
        let cwd = s.cwd.as_deref().unwrap_or("?");
        let sid = s.session_id.as_deref().unwrap_or("?");
        let updated = s.updated_at.map_or("unknown".to_string(), |t| {
            format_time(t)
        });

        println!(
            "PID: {:8}  Status: {GREEN}{:8}{RESET}  CWD: {}",
            pid, status, cwd
        );
        println!(
            "  Session: {BOLD}{}{RESET}  Updated: {}",
            sid, updated
        );
    }
}

// ── interactive prompt ───────────────────────────────────────────

fn prompt(question: &str) -> String {
    print!("{}", question);
    io::stdout().flush().ok();
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).ok();
    line.trim().to_string()
}
