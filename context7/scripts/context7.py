#!/usr/bin/env python3
"""
Retrieve up-to-date documentation for software libraries via the Context7 API.

Usage:
    python3 context7.py search <library> <topic>
    python3 context7.py fetch <library_id> <topic> [--max-tokens N]

Subcommands:
    search    Find a library ID by name and topic
    fetch     Retrieve documentation snippets for a library

Arguments:
    library      Library name to search for (e.g., "react", "nextjs", "fastapi")
    library_id   Library ID from search results (e.g., "/websites/react_dev")
    topic        Specific question or topic (e.g., "useState hook", "app router middleware")

Options:
    --max-tokens N   Max tokens to return (default: 5000). Fetches JSON and
                     truncates to budget, keeping most relevant snippets first.

Environment:
    CONTEXT7_API_KEY    Required. API key for Context7.

Tips:
    Write specific queries for better results:
      Good:  "How to use useState hook with objects"
      Bad:   "hooks"

Examples:
    python3 context7.py search react "useState hook"
    python3 context7.py fetch /websites/react_dev "useState hook with objects"
    python3 context7.py fetch /websites/react_dev "server components" --max-tokens 3000
"""

import json
import os
import sys
import urllib.parse
import urllib.request

API_BASE = "https://context7.com/api/v2"


def fail(msg):
    print(f"Error: {msg}", file=sys.stderr)
    sys.exit(1)


def usage():
    print(__doc__.strip())


def require_api_key():
    key = os.environ.get("CONTEXT7_API_KEY", "")
    if not key:
        fail("CONTEXT7_API_KEY environment variable is not set.")
    return key


def api_get(path, api_key):
    url = f"{API_BASE}/{path}"
    req = urllib.request.Request(url, headers={"Authorization": f"Bearer {api_key}"})
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            return json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        fail(f"API request failed: HTTP {e.code}")
    except urllib.error.URLError as e:
        fail(f"API request failed: {e.reason}")


def do_search(library, topic, api_key):
    path = (
        f"libs/search?libraryName={urllib.parse.quote(library)}"
        f"&query={urllib.parse.quote(topic)}"
    )
    data = api_get(path, api_key)

    print("id\ttitle\tsnippets")
    for r in data.get("results", [])[:5]:
        print(f"{r['id']}\t{r['title']}\t{r.get('totalSnippets', 0)}")


def do_fetch(library_id, topic, max_tokens, api_key):
    path = (
        f"context?libraryId={urllib.parse.quote(library_id)}"
        f"&query={urllib.parse.quote(topic)}&type=json"
    )
    data = api_get(path, api_key)

    budget = max_tokens
    used = 0

    # Code snippets (ordered by relevance from API)
    for s in data.get("codeSnippets", []):
        tokens = s.get("codeTokens", 0)
        if used + tokens > budget:
            break
        title = s.get("codeTitle", "")
        desc = s.get("codeDescription", "")
        if title:
            print(f"### {title}")
        if desc:
            print(desc)
            print()
        for block in s.get("codeList", []):
            lang = block.get("language", "")
            code = block.get("code", "")
            print(f"```{lang}")
            print(code)
            print("```")
        print()
        print("---")
        print()
        used += tokens

    # Info snippets with remaining budget
    for s in data.get("infoSnippets", []):
        tokens = s.get("contentTokens", 0)
        if used + tokens > budget:
            break
        content = s.get("content", "")
        if content:
            print(content)
            print()
            print("---")
            print()
        used += tokens

    print(f"[{used} tokens used, budget {budget}]", file=sys.stderr)


def main():
    api_key = require_api_key()

    if len(sys.argv) < 2:
        usage()
        sys.exit(1)

    subcommand = sys.argv[1]

    if subcommand in ("-h", "--help", "help"):
        usage()
        sys.exit(0)

    if subcommand == "search":
        if len(sys.argv) < 4:
            fail("Usage: context7.py search <library> <topic>")
        do_search(sys.argv[2], sys.argv[3], api_key)

    elif subcommand == "fetch":
        if len(sys.argv) < 4:
            fail("Usage: context7.py fetch <library_id> <topic> [--max-tokens N]")
        library_id = sys.argv[2]
        topic = sys.argv[3]
        max_tokens = 5000
        i = 4
        while i < len(sys.argv):
            if sys.argv[i] == "--max-tokens":
                if i + 1 >= len(sys.argv):
                    fail("--max-tokens requires a value")
                max_tokens = int(sys.argv[i + 1])
                i += 2
            else:
                fail(f"Unknown option: {sys.argv[i]}")
        do_fetch(library_id, topic, max_tokens, api_key)

    else:
        fail(f"Unknown subcommand '{subcommand}'. Use: search, fetch.")


if __name__ == "__main__":
    main()
