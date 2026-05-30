mod sessions;
mod utils;

use sessions::*;
use utils::*;

struct CliOptions {
    project: Option<String>,
    limit: Option<usize>,
    json: bool,
    force: bool,
    all: bool,
}

fn parse_args(args: &[String]) -> (Vec<String>, CliOptions) {
    let mut positional = Vec::new();
    let mut opts = CliOptions {
        project: None,
        limit: None,
        json: false,
        force: false,
        all: false,
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--project" => {
                if i + 1 < args.len() {
                    i += 1;
                    opts.project = Some(args[i].clone());
                }
            }
            "--limit" => {
                if i + 1 < args.len() {
                    i += 1;
                    opts.limit = args[i].parse().ok();
                }
            }
            "--json" => opts.json = true,
            "--force" => opts.force = true,
            "--all" => opts.all = true,
            _ => positional.push(args[i].clone()),
        }
        i += 1;
    }

    (positional, opts)
}

fn usage() {
    println!(
        "Usage: claude-sessions <command> [options]

{BOLD}Commands:{RESET}
  list, ls              List all sessions
    --project <name>    Filter by project name (fuzzy match)
    --limit N           Limit results to N most recent
    --json              Output as JSON Lines
  show <id>             Show detailed session info
  delete <id> [--force] Delete a session and all associated files
  clear [--force]       Clear all sessions, or --project <name> for one project
  projects              List projects with session counts
  active, running       Show currently active sessions
  help                  Print this message"
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str);
    let rest: Vec<String> = args.iter().skip(2).cloned().collect();
    let (positional, opts) = parse_args(&rest);

    match cmd {
        Some("list") | Some("ls") => {
            list_sessions(opts.project.as_deref(), opts.limit, opts.json, opts.all);
        }
        Some("show") | Some("info") | Some("inspect") => {
            if positional.is_empty() {
                eprintln!("Usage: claude-sessions show <session-id>");
                std::process::exit(1);
            }
            show_session(&positional[0]);
        }
        Some("delete") | Some("rm") | Some("remove") => {
            if positional.is_empty() {
                eprintln!("Usage: claude-sessions delete <session-id> [--force]");
                std::process::exit(1);
            }
            remove_session(&positional[0], opts.force);
        }
        Some("clear") => {
            clear_sessions(opts.project.as_deref(), opts.force);
        }
        Some("projects") | Some("prj") => {
            list_projects();
        }
        Some("active") | Some("running") => {
            list_active();
        }
        Some("help") | Some("-h") | Some("--help") => {
            usage();
        }
        None => {
            list_sessions(None, Some(20), false, false);
        }
        _ => {
            list_sessions(None, Some(20), false, false);
        }
    }
}
