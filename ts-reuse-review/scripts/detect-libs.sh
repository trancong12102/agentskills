#!/usr/bin/env bash
# detect-libs.sh — scan package.json(s) in the repo for reuse-relevant libs.
# Output: NDJSON, one line per installed relevant lib.
# Exit 0 even when no libs found (empty stdout).

set -euo pipefail

ROOT="${1:-.}"

usage() {
    cat <<'USAGE'
Usage: detect-libs.sh [ROOT]

Scans ROOT/package.json and any ROOT/packages/*/package.json (monorepo) for
TypeScript/JavaScript libraries relevant to reuse review.

Output: NDJSON, one line per detected lib, e.g.
  {"lib":"es-toolkit","group":"general","version":"1.21.0","source":"package.json"}

Groups: general, date, schema, async, http, collection, state
USAGE
}

case "${1:-}" in
-h | --help)
    usage
    exit 0
    ;;
esac

if [[ ! -d "$ROOT" ]]; then
    echo "ROOT not a directory: $ROOT" >&2
    exit 2
fi

# group map — edit here to add/remove tracked libs
declare -a RELEVANT=(
    # general utils
    "es-toolkit:general"
    "lodash:general"
    "lodash-es:general"
    "ramda:general"
    "remeda:general"
    "radash:general"
    # date
    "date-fns:date"
    "dayjs:date"
    "luxon:date"
    "moment:date"
    # schema
    "zod:schema"
    "valibot:schema"
    "yup:schema"
    "superstruct:schema"
    "arktype:schema"
    "runtypes:schema"
    "@effect/schema:schema"
    # async / effects
    "effect:async"
    "neverthrow:async"
    "ts-pattern:async"
    "rxjs:async"
    "p-retry:async"
    "p-queue:async"
    "p-limit:async"
    "p-map:async"
    # http / query
    "ky:http"
    "ofetch:http"
    "@tanstack/react-query:http"
    "@tanstack/query-core:http"
    "swr:http"
    # collection / immutability
    "immer:collection"
    "immutable:collection"
    # state
    "zustand:state"
    "jotai:state"
    "xstate:state"
)

# Collect package.json files: root + packages/* + apps/*  (bash 3-compatible, no mapfile)
PKGFILES=()
while IFS= read -r LINE; do
    [[ -n "$LINE" ]] && PKGFILES+=("$LINE")
done < <(
    find "$ROOT" -maxdepth 1 -name package.json -type f 2>/dev/null
    find "$ROOT/packages" -maxdepth 2 -name package.json -type f 2>/dev/null || true
    find "$ROOT/apps" -maxdepth 2 -name package.json -type f 2>/dev/null || true
)

if [[ ${#PKGFILES[@]} -eq 0 ]]; then
    exit 0
fi

for PKG in "${PKGFILES[@]}"; do
    REL="${PKG#$ROOT/}"
    for ENTRY in "${RELEVANT[@]}"; do
        LIB="${ENTRY%:*}"
        GROUP="${ENTRY##*:}"
        # check deps + devDeps + peerDeps via node/jq if available, else grep
        VERSION=""
        if command -v jq >/dev/null 2>&1; then
            VERSION=$(jq -r --arg L "$LIB" '
                (.dependencies // {})[$L]
                // (.devDependencies // {})[$L]
                // (.peerDependencies // {})[$L]
                // empty
            ' "$PKG" 2>/dev/null || true)
        else
            # crude fallback: find line like "lib": "version"
            VERSION=$(grep -E "\"$LIB\"\\s*:\\s*\"[^\"]+\"" "$PKG" 2>/dev/null | head -n 1 | sed -E 's/.*"([^"]+)"\s*$/\1/' || true)
        fi
        if [[ -n "$VERSION" ]]; then
            printf '{"lib":"%s","group":"%s","version":"%s","source":"%s"}\n' \
                "$LIB" "$GROUP" "$VERSION" "$REL"
        fi
    done
done
