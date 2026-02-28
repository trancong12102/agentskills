#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  context7.sh search <library> <topic>
  context7.sh fetch <library_id> <topic> [--max-tokens N]

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
  context7.sh search react "useState hook"
  context7.sh fetch /websites/react_dev "useState hook with objects"
  context7.sh fetch /websites/react_dev "server components" --max-tokens 3000
EOF
}

fail() {
  echo "Error: $*" >&2
  exit 1
}

require_api_key() {
  if [[ -z "${CONTEXT7_API_KEY:-}" ]]; then
    fail "CONTEXT7_API_KEY environment variable is not set."
  fi
}

urlencode() {
  python3 -c "import urllib.parse,sys; print(urllib.parse.quote(sys.argv[1]))" "$1"
}

API_BASE="https://context7.com/api/v2"

do_search() {
  local library="$1"
  local topic="$2"

  local response
  response="$(curl -sf -H "Authorization: Bearer $CONTEXT7_API_KEY" \
    "${API_BASE}/libs/search?libraryName=$(urlencode "$library")&query=$(urlencode "$topic")")" \
    || fail "Search request failed."

  # Output compact TSV: id, title, snippets
  echo "id	title	snippets"
  echo "$response" | python3 -c "
import sys, json
data = json.load(sys.stdin)
for r in data.get('results', [])[:5]:
    print(f\"{r['id']}\t{r['title']}\t{r.get('totalSnippets', 0)}\")
"
}

do_fetch() {
  local library_id="$1"
  local topic="$2"
  local max_tokens="${3:-5000}"

  # Fetch JSON to get token counts per snippet for truncation
  local response
  response="$(curl -sf -H "Authorization: Bearer $CONTEXT7_API_KEY" \
    "${API_BASE}/context?libraryId=$(urlencode "$library_id")&query=$(urlencode "$topic")&type=json")" \
    || fail "Fetch request failed."

  # Truncate to token budget, output plain text
  echo "$response" | python3 -c "
import sys, json

data = json.load(sys.stdin)
budget = int(sys.argv[1])
used = 0

# Code snippets (ordered by relevance from API)
for s in data.get('codeSnippets', []):
    tokens = s.get('codeTokens', 0)
    if used + tokens > budget:
        break
    title = s.get('codeTitle', '')
    desc = s.get('codeDescription', '')
    if title:
        print(f'### {title}')
    if desc:
        print(desc)
        print()
    for block in s.get('codeList', []):
        lang = block.get('language', '')
        code = block.get('code', '')
        print(f'\`\`\`{lang}')
        print(code)
        print('\`\`\`')
    print()
    print('---')
    print()
    used += tokens

# Info snippets with remaining budget
for s in data.get('infoSnippets', []):
    tokens = s.get('contentTokens', 0)
    if used + tokens > budget:
        break
    content = s.get('content', '')
    if content:
        print(content)
        print()
        print('---')
        print()
    used += tokens

print(f'[{used} tokens used, budget {budget}]', file=sys.stderr)
" "$max_tokens"
}

main() {
  require_api_key
  [[ $# -gt 0 ]] || { usage; exit 1; }

  local subcommand="$1"; shift

  case "$subcommand" in
    search)
      [[ $# -ge 2 ]] || fail "Usage: context7.sh search <library> <topic>"
      do_search "$1" "$2"
      ;;
    fetch)
      [[ $# -ge 2 ]] || fail "Usage: context7.sh fetch <library_id> <topic> [--max-tokens N]"
      local library_id="$1"
      local topic="$2"
      shift 2
      local max_tokens=5000
      while [[ $# -gt 0 ]]; do
        case "$1" in
          --max-tokens) max_tokens="$2"; shift 2 ;;
          *) fail "Unknown option: $1" ;;
        esac
      done
      do_fetch "$library_id" "$topic" "$max_tokens"
      ;;
    -h|--help|help) usage ;;
    *) fail "Unknown subcommand '$subcommand'. Use: search, fetch." ;;
  esac
}

main "$@"
