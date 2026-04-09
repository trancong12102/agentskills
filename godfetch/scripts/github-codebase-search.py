#!/usr/bin/env python3
"""
Semantic search for public GitHub repos via MorphLLM (through mcporter).

Usage:
    python3 github-codebase-search.py search <query> --repo <owner/repo> [--branch B] [--timeout N] [--dry-run]
    python3 github-codebase-search.py search <query> --url <github_url> [--branch B] [--timeout N] [--dry-run]

Subcommands:
    search    Run a semantic search query against a GitHub repository

Arguments:
    query         Natural language question about the code (e.g., "how does routing work")

Options:
    --repo OWNER/REPO    GitHub repository in owner/repo format (e.g., "vercel/next.js")
    --url URL            GitHub URL (e.g., "https://github.com/vercel/next.js")
    --branch BRANCH      Branch to search (defaults to repo's default branch)
    --timeout N          Timeout in seconds for mcporter call (default: 120)
    --dry-run            Print the mcporter command without executing it

    Must provide either --repo or --url.

Environment:
    MORPH_API_KEY    Required. API key for MorphLLM.

Tips:
    Write queries as natural language questions — the search agent runs parallel
    grep+read calls against the GitHub API to answer them:
      Good:  "How does the router resolve middleware chains?"
      Good:  "How does Prisma handle relation loading in findMany?"
      Bad:   "router" (too vague — be specific about what you want to understand)

Examples:
    python3 github-codebase-search.py search "how does routing work" --repo vercel/next.js
    python3 github-codebase-search.py search "how does routing work" --url https://github.com/vercel/next.js
    python3 github-codebase-search.py search "how are migrations handled" --repo prisma/prisma --branch main
    python3 github-codebase-search.py search "trace the build pipeline" --repo facebook/react --timeout 180
    python3 github-codebase-search.py search "how does routing work" --repo vercel/next.js --dry-run
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


def has_configured_server(mcporter):
    """Check if morphmcp is configured as a named server in mcporter."""
    try:
        result = subprocess.run(
            mcporter + ["config", "list", "--json"],
            capture_output=True, text=True, timeout=5,
        )
        return '"morphmcp"' in result.stdout
    except Exception:
        return False


def do_search(query, owner_repo, github_url, branch, timeout, dry_run, api_key):
    mcporter = find_mcporter()

    # Use configured server if available (faster with daemon keep-alive),
    # otherwise fall back to ad-hoc --stdio
    if has_configured_server(mcporter):
        cmd = mcporter + [
            "call",
            "morphmcp.github_codebase_search",
            f"search_string={query}",
        ]
    else:
        cmd = mcporter + [
            "call",
            "--stdio", "bunx @morphllm/morphmcp@latest",
            "--env", f"MORPH_API_KEY={api_key}",
            "github_codebase_search",
            f"search_string={query}",
        ]

    if owner_repo:
        cmd.append(f"owner_repo={owner_repo}")
    if github_url:
        cmd.append(f"github_url={github_url}")
    if branch:
        cmd.append(f"branch={branch}")

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
            fail("Usage: github-codebase-search.py search <query> --repo <owner/repo> [--branch B] [--timeout N] [--dry-run]")

        query = sys.argv[2]
        owner_repo = None
        github_url = None
        branch = None
        timeout = 120
        dry_run = False

        i = 3
        while i < len(sys.argv):
            arg = sys.argv[i]
            if arg == "--repo":
                if i + 1 >= len(sys.argv):
                    fail("--repo requires a value")
                owner_repo = sys.argv[i + 1]
                i += 2
            elif arg == "--url":
                if i + 1 >= len(sys.argv):
                    fail("--url requires a value")
                github_url = sys.argv[i + 1]
                i += 2
            elif arg == "--branch":
                if i + 1 >= len(sys.argv):
                    fail("--branch requires a value")
                branch = sys.argv[i + 1]
                i += 2
            elif arg == "--timeout":
                if i + 1 >= len(sys.argv):
                    fail("--timeout requires a value")
                timeout = int(sys.argv[i + 1])
                i += 2
            elif arg == "--dry-run":
                dry_run = True
                i += 1
            else:
                fail(f"Unknown option: {arg}")

        if not owner_repo and not github_url:
            fail("Must provide either --repo <owner/repo> or --url <github_url>")
        if owner_repo and github_url:
            fail("Provide --repo or --url, not both")
        if owner_repo and "/" not in owner_repo:
            fail(f"--repo must be in owner/repo format (e.g., 'vercel/next.js'), got: {owner_repo}")

        api_key = require_api_key()
        do_search(query, owner_repo, github_url, branch, timeout, dry_run, api_key)

    else:
        fail(f"Unknown subcommand '{subcommand}'. Use: search.")


if __name__ == "__main__":
    main()
