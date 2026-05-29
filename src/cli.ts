#!/usr/bin/env bun
/**
 * claude-sessions — standalone CLI
 *
 * Usage: claude-sessions <command> [options]
 */

import { list, show, remove, projects, active } from './sessions.js';
import { bold, reset } from './utils.js';

interface CliOptions {
  project?: string;
  limit?: number;
  json?: boolean;
  force?: boolean;
  all?: boolean;
}

function parseArgs(argv: string[]): { args: string[]; opts: CliOptions } {
  const args: string[] = [];
  const opts: CliOptions = {};

  for (let i = 0; i < argv.length; i++) {
    switch (argv[i]) {
      case '--project':
        if (i + 1 < argv.length) opts.project = argv[++i];
        break;
      case '--limit':
        if (i + 1 < argv.length) opts.limit = parseInt(argv[++i], 10);
        break;
      case '--json':
        opts.json = true;
        break;
      case '--force':
        opts.force = true;
        break;
      case '--all':
        opts.all = true;
        break;
      default:
        args.push(argv[i]);
    }
  }
  return { args, opts };
}

function usage(): void {
  console.log(`Usage: claude-sessions <command> [options]

${bold}Commands:${reset}
  list, ls              List all sessions
    --project <name>    Filter by project name (fuzzy match)
    --limit N           Limit results to N most recent
    --json              Output as JSON Lines
  show <id>             Show detailed session info
  delete <id> [--force] Delete a session and all associated files
  projects              List projects with session counts
  active, running       Show currently active sessions
  help                  Print this message`);
}

function main(): void {
  const cmd = process.argv[2];
  const { args, opts } = parseArgs(process.argv.slice(3));

  switch (cmd) {
    case 'list':
    case 'ls':
      list({ project: opts.project, limit: opts.limit, json: opts.json, all: opts.all });
      break;
    case 'show':
    case 'info':
    case 'inspect':
      if (!args[0]) {
        console.error('Usage: claude-sessions show <session-id>');
        process.exit(1);
      }
      show(args[0]);
      break;
    case 'delete':
    case 'rm':
    case 'remove':
      if (!args[0]) {
        console.error('Usage: claude-sessions delete <session-id> [--force]');
        process.exit(1);
      }
      remove(args[0], { force: opts.force });
      break;
    case 'projects':
    case 'prj':
      projects();
      break;
    case 'active':
    case 'running':
      active();
      break;
    case 'help':
    case '-h':
    case '--help':
      usage();
      break;
    default:
      // Default to list (current project)
      list({ all: opts.all, limit: 20 });
      break;
  }
}

main();
