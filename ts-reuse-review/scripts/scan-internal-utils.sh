#!/usr/bin/env bash
# scan-internal-utils.sh — search workspace util dirs for existing helpers.
# Input: candidate function names (positional args).
# Output: NDJSON, one line per match.
# Exit 0 even when nothing found.

set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: scan-internal-utils.sh [--root DIR] NAME [NAME...]

Search workspace utility directories for function exports matching candidate names.

Match forms:
  export function <NAME>(...)
  export const <NAME> = ...
  export { <NAME> }
  export { foo as <NAME> }

Searched paths (relative to --root, default "."):
  src/**/utils/**, src/**/lib/**, src/**/helpers/**
  packages/*/src/**, apps/*/src/**
  shared/**, common/**
  root: utils.{ts,js}, helpers.{ts,js}, lib.{ts,js}

Output: NDJSON, e.g.
  {"name":"chunk","path":"src/utils/array.ts","line":12,"kind":"function"}

Exit 0 even if no matches.
USAGE
}

ROOT="."
if [[ "${1:-}" == "--root" ]]; then
    ROOT="$2"
    shift 2
fi

case "${1:-}" in
-h | --help | "")
    usage
    [[ -z "${1:-}" ]] && exit 2
    exit 0
    ;;
esac

if ! command -v rg >/dev/null 2>&1; then
    echo "ripgrep (rg) not found — install: brew install ripgrep" >&2
    exit 3
fi

NAMES=("$@")

# build rg alternation: name1|name2|name3
PATTERN=""
for N in "${NAMES[@]}"; do
    [[ -n "$PATTERN" ]] && PATTERN="$PATTERN|"
    PATTERN="$PATTERN$N"
done

# ripgrep globs for util-ish directories
GLOBS=(
    -g "src/**/utils/**/*.{ts,tsx,js,jsx,mjs,cjs}"
    -g "src/**/lib/**/*.{ts,tsx,js,jsx,mjs,cjs}"
    -g "src/**/helpers/**/*.{ts,tsx,js,jsx,mjs,cjs}"
    -g "packages/*/src/**/*.{ts,tsx,js,jsx,mjs,cjs}"
    -g "apps/*/src/**/*.{ts,tsx,js,jsx,mjs,cjs}"
    -g "shared/**/*.{ts,tsx,js,jsx,mjs,cjs}"
    -g "common/**/*.{ts,tsx,js,jsx,mjs,cjs}"
    -g "utils.{ts,js,mjs,cjs}"
    -g "helpers.{ts,js,mjs,cjs}"
    -g "lib.{ts,js,mjs,cjs}"
    -g "!**/node_modules/**"
    -g "!**/dist/**"
    -g "!**/build/**"
    -g "!**/.next/**"
    -g "!**/*.d.ts"
)

# regex matches: function, const-arrow, named export
REGEX="export[[:space:]]+function[[:space:]]+($PATTERN)\\b|\
export[[:space:]]+const[[:space:]]+($PATTERN)[[:space:]]*=|\
export[[:space:]]*\\{[^}]*\\b($PATTERN)\\b[^}]*\\}|\
export[[:space:]]*\\{[^}]*\\bas[[:space:]]+($PATTERN)\\b[^}]*\\}"

cd "$ROOT"

{ rg --json --multiline "${GLOBS[@]}" -e "$REGEX" 2>/dev/null || true; } | while IFS= read -r LINE; do
    # only look at "match" events
    [[ "$LINE" != *'"type":"match"'* ]] && continue

    # Extract path, line_number, line text via python for robustness
    python3 -c '
import json, sys, re
data = json.loads(sys.argv[1])
if data.get("type") != "match":
    sys.exit(0)
d = data["data"]
path = d["path"]["text"]
base_line = d["line_number"]
text = d["lines"]["text"]
names_re = re.compile(r"export\s+function\s+(\w+)|export\s+const\s+(\w+)\s*=|export\s*\{([^}]*)\}")
for m in names_re.finditer(text):
    offset = text[:m.start()].count("\n")
    line_num = base_line + offset
    if m.group(1):
        print(json.dumps({"name": m.group(1), "path": path, "line": line_num, "kind": "function"}))
    elif m.group(2):
        print(json.dumps({"name": m.group(2), "path": path, "line": line_num, "kind": "const-arrow"}))
    elif m.group(3):
        inside = m.group(3)
        for piece in inside.split(","):
            piece = piece.strip()
            if " as " in piece:
                _, name = piece.split(" as ", 1)
                name = name.strip()
            else:
                name = piece
            if name:
                print(json.dumps({"name": name, "path": path, "line": line_num, "kind": "named-export"}))
' "$LINE" 2>/dev/null || true
done
