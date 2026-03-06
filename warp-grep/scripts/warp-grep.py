#!/usr/bin/env python3
"""
Semantic codebase search via MorphLLM's warpgrep (through mcporter).

Usage:
    python3 warp-grep.py search <query> [repo_path] [--search-type TYPE] [--timeout N] [--dry-run]

Subcommands:
    search    Run a semantic search query against a codebase

Arguments:
    query         Natural language question about the code (e.g., "how does auth work")
    repo_path     Path to repository root (default: current directory)

Options:
    --search-type TYPE   Search type: "default" or "node_modules" (default: "default")
    --timeout N          Timeout in seconds for mcporter call (default: 120)
    --dry-run            Print the mcporter command without executing it

Environment:
    MORPH_API_KEY    Required. API key for MorphLLM.

Tips:
    Write queries as natural language questions — warpgrep is an RL-trained
    subagent that runs ~15-30 internal grep+read ops to answer them:
      Good:  "How does the authentication middleware validate JWT tokens?"
      Good:  "What happens when a user submits a form on the settings page?"
      Bad:   "auth" (too vague — use regular grep for simple keyword searches)

Examples:
    python3 warp-grep.py search "how does auth work"
    python3 warp-grep.py search "how does auth work" /path/to/repo
    python3 warp-grep.py search "where are deps resolved" --search-type node_modules
    python3 warp-grep.py search "trace the payment flow" --timeout 180
    python3 warp-grep.py search "how does auth work" --dry-run
"""

import os
import shutil
import subprocess
import sys


def fail(msg):
    print(f"Error: {msg}", file=sys.stderr)
    sys.exit(1)


def usage():
    print(__doc__.strip())


def require_api_key():
    key = os.environ.get("MORPH_API_KEY", "")
    if not key:
        fail("MORPH_API_KEY environment variable is not set.")
    return key


def find_mcporter():
    """Find mcporter binary — prefer global install over bunx for speed."""
    path = shutil.which("mcporter")
    if path:
        return [path]
    return ["bunx", "mcporter"]


def has_configured_server():
    """Check if morphmcp is configured as a named server in mcporter."""
    mcporter = find_mcporter()
    try:
        result = subprocess.run(
            mcporter + ["config", "list", "--json"],
            capture_output=True, text=True, timeout=5,
        )
        return '"morphmcp"' in result.stdout
    except Exception:
        return False


def do_search(query, repo_path, search_type, timeout, dry_run, api_key):
    repo_path = os.path.abspath(repo_path)
    if not os.path.isdir(repo_path):
        fail(f"repo_path is not a directory: {repo_path}")

    mcporter = find_mcporter()

    # Use configured server if available (faster with daemon keep-alive),
    # otherwise fall back to ad-hoc --stdio
    if has_configured_server():
        cmd = mcporter + [
            "call",
            "morphmcp.warpgrep_codebase_search",
            f"search_string={query}",
            f"repo_path={repo_path}",
        ]
    else:
        cmd = mcporter + [
            "call",
            "--stdio", "bunx @morphllm/morphmcp@latest",
            "--env", f"MORPH_API_KEY={api_key}",
            "--env", "ENABLED_TOOLS=warpgrep_codebase_search",
            "warpgrep_codebase_search",
            f"search_string={query}",
            f"repo_path={repo_path}",
        ]
    if search_type != "default":
        cmd.append(f"search_type={search_type}")

    if dry_run:
        display_cmd = []
        for arg in cmd:
            if arg.startswith("MORPH_API_KEY="):
                display_cmd.append("MORPH_API_KEY=****")
            else:
                display_cmd.append(arg)
        print(" ".join(display_cmd))
        return

    try:
        result = subprocess.run(cmd, capture_output=True, timeout=timeout)
    except FileNotFoundError:
        fail("mcporter not found. Install: bun install -g mcporter@latest")
    except subprocess.TimeoutExpired:
        fail(f"Command timed out after {timeout}s. Try increasing --timeout.")

    stdout = result.stdout.decode("utf-8", errors="replace") if result.stdout else ""
    stderr = result.stderr.decode("utf-8", errors="replace") if result.stderr else ""
    if stdout:
        print(stdout, end="")
    if stderr:
        print(stderr, end="", file=sys.stderr)
    if result.returncode != 0:
        sys.exit(result.returncode)


def main():
    if len(sys.argv) < 2:
        usage()
        sys.exit(1)

    subcommand = sys.argv[1]

    if subcommand in ("-h", "--help", "help"):
        usage()
        sys.exit(0)

    if subcommand == "search":
        if len(sys.argv) < 3:
            fail("Usage: warp-grep.py search <query> [repo_path] [--search-type TYPE] [--timeout N] [--dry-run]")

        query = sys.argv[2]
        repo_path = os.getcwd()
        search_type = "default"
        timeout = 120
        dry_run = False

        i = 3
        while i < len(sys.argv):
            arg = sys.argv[i]
            if arg == "--search-type":
                if i + 1 >= len(sys.argv):
                    fail("--search-type requires a value")
                search_type = sys.argv[i + 1]
                if search_type not in ("default", "node_modules"):
                    fail(f"Invalid search type: {search_type}. Use: default, node_modules")
                i += 2
            elif arg == "--timeout":
                if i + 1 >= len(sys.argv):
                    fail("--timeout requires a value")
                timeout = int(sys.argv[i + 1])
                i += 2
            elif arg == "--dry-run":
                dry_run = True
                i += 1
            elif not arg.startswith("-") and i == 3:
                repo_path = arg
                i += 1
            else:
                fail(f"Unknown option: {arg}")

        api_key = require_api_key()
        do_search(query, repo_path, search_type, timeout, dry_run, api_key)

    else:
        fail(f"Unknown subcommand '{subcommand}'. Use: search.")


if __name__ == "__main__":
    main()
