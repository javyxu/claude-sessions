use serde::de::DeserializeOwned;
use std::path::Path;

// ── terminal colors ─────────────────────────────────────────────

pub const BOLD: &str = "\x1b[1m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const RESET: &str = "\x1b[0m";

// ── formatting ──────────────────────────────────────────────────

pub fn human_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1048576 {
        format!("{}K", bytes / 1024)
    } else {
        format!("{:.1}M", bytes as f64 / 1048576.0)
    }
}

pub fn format_time(ms: i64) -> String {
    if ms <= 0 {
        return "unknown".to_string();
    }
    let secs = ms / 1000;
    // Format as YYYY-MM-DD HH:MM:SS using simple arithmetic
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Convert days since Unix epoch to date
    let (y, m, d) = days_to_date(days as i64);

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y, m, d, hours, minutes, seconds
    )
}

fn days_to_date(mut days: i64) -> (i64, u32, u32) {
    // Days since 1970-01-01
    let mut year = 1970i64;

    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    static MONTH_DAYS: [[i64; 12]; 2] = [
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    ];
    let leap_idx = if is_leap(year) { 1 } else { 0 };
    let mut month = 1u32;
    for &md in &MONTH_DAYS[leap_idx] {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }

    (year, month, (days + 1) as u32)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

// ── path encoding ───────────────────────────────────────────────

pub fn encode_project(cwd: &str) -> String {
    let absolute = if cwd.starts_with('/') {
        cwd.to_string()
    } else {
        format!("/{}", cwd)
    };
    format!("-{}", absolute[1..].replace('/', "-"))
}

pub fn decode_project(encoded: &str) -> String {
    let decoded = if let Some(rest) = encoded.strip_prefix('-') {
        format!("/{}", rest.replace('-', "/"))
    } else {
        encoded.to_string()
    };

    // Validate against filesystem
    if Path::new(&decoded).exists() {
        return decoded;
    }

    // Try finding the longest valid prefix
    let parts: Vec<&str> = decoded.split('/').collect();
    for i in (1..=parts.len()).rev() {
        let candidate = parts[..i].join("/");
        let candidate = if candidate.is_empty() { "/".to_string() } else { candidate };
        if Path::new(&candidate).exists() {
            return format!("{} [?]", decoded);
        }
    }

    format!("{} [?]", decoded)
}

// ── safe JSON ───────────────────────────────────────────────────

pub fn safe_json<T: DeserializeOwned + Default>(s: &str) -> T {
    serde_json::from_str(s).unwrap_or_default()
}
