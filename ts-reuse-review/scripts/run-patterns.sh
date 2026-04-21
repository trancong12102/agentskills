#!/usr/bin/env bash
# run-patterns.sh — run all ast-grep rules over one or more TS/JS/TSX/JSX files.
# Handles language auto-switch: rules live as `language: TypeScript` but are
# replayed with `language: Tsx` for .tsx/.jsx files (different parser).

set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: run-patterns.sh FILE [FILE...]

Runs every rule in scripts/patterns/ against each FILE.
Output from ast-grep goes to stdout. Non-TS/JS files are skipped silently.

Requires: ast-grep on PATH (npm i -g @ast-grep/cli).
USAGE
}

case "${1:-}" in
-h | --help | "")
    usage
    [[ -z "${1:-}" ]] && exit 2
    exit 0
    ;;
esac

if ! command -v ast-grep >/dev/null 2>&1; then
    echo "ast-grep not found — install: npm i -g @ast-grep/cli" >&2
    exit 3
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RULE_DIR="$SCRIPT_DIR/patterns"

if [[ ! -d "$RULE_DIR" ]]; then
    echo "patterns/ dir missing: $RULE_DIR" >&2
    exit 4
fi

TMP_DIR=$(mktemp -d 2>/dev/null || mktemp -d -t 'tsrr')
trap 'rm -rf "$TMP_DIR"' EXIT

# Pre-build Tsx variants (once) — same content, language swapped
for rule in "$RULE_DIR"/*.yml; do
    sed 's/^language: TypeScript$/language: Tsx/' "$rule" >"$TMP_DIR/$(basename "$rule")"
done

for file in "$@"; do
    [[ ! -f "$file" ]] && continue
    ext="${file##*.}"
    case "$ext" in
    tsx | jsx)
        ACTIVE_DIR="$TMP_DIR"
        ;;
    ts | js | mjs | cjs)
        ACTIVE_DIR="$RULE_DIR"
        ;;
    *)
        continue
        ;;
    esac
    for rule in "$ACTIVE_DIR"/*.yml; do
        ast-grep scan -r "$rule" "$file" 2>/dev/null || true
    done
done
