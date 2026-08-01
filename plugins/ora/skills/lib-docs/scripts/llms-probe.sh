#!/usr/bin/env bash
# Probe a docs site for llms.txt / llms-full.txt.
#
# Usage:
#   llms-probe.sh <domain-or-url>
#
# Output (TSV): kind \t url \t size
#   kind: "index" (llms.txt) or "full" (llms-full.txt)
#   size: human-readable (e.g. 478KB, 4.8MB) or "?" if no Content-Length
#
# Examples:
#   llms-probe.sh react.dev
#   llms-probe.sh https://docs.cloudflare.com
#   llms-probe.sh nextjs.org
#
# Probes (HEAD, follows redirects, 8s timeout):
#   /llms.txt, /llms-full.txt
#   /docs/llms.txt, /docs/llms-full.txt
#   /en/llms.txt, /en/llms-full.txt
#
# Filters: status 200 + non-HTML content-type (some sites soft-404 with HTML).
# Exit: 0 if any file found, 1 otherwise.

set -euo pipefail

input="${1:-}"
case "$input" in
  ""|-h|--help)
    sed -n '2,/^$/p' "$0" | sed 's/^# \{0,1\}//'
    [[ -z "$input" ]] && exit 1 || exit 0 ;;
esac

# Extract host from input: strip scheme, path, trailing slash.
host="${input#http://}"
host="${host#https://}"
host="${host%%/*}"

paths=(
  /llms.txt
  /llms-full.txt
  /docs/llms.txt
  /docs/llms-full.txt
  /en/llms.txt
  /en/llms-full.txt
)

human_size() {
  local n="${1:-0}"
  if [[ -z "$n" || "$n" == "0" ]]; then
    echo "?"
  elif (( n < 1024 )); then
    echo "${n}B"
  elif (( n < 1048576 )); then
    echo "$(( n / 1024 ))KB"
  else
    awk -v n="$n" 'BEGIN{printf "%.1fMB", n/1048576}'
  fi
}

declare -A seen=()
found=0
for path in "${paths[@]}"; do
  url="https://${host}${path}"

  # HEAD with redirect follow; capture status + final URL + content-type.
  meta=$(curl -sILk --max-time 8 \
    -w "%{http_code}\t%{content_type}\t%{url_effective}" \
    -o /dev/null "$url" 2>/dev/null || printf "0\t\t%s" "$url")

  IFS=$'\t' read -r status ctype final <<<"$meta"

  [[ "$status" != "200" ]] && continue
  [[ "$ctype" == *"text/html"* ]] && continue
  [[ -n "${seen[$final]:-}" ]] && continue
  seen[$final]=1

  # Try Content-Length from HEAD first; if absent, try a 1-byte range GET
  # and parse Content-Range. Some CDNs strip both — size stays "?".
  # `|| true` guards `set -e` against curl timeout (exit 28).
  len=$( { curl -sIk --max-time 5 "$final" 2>/dev/null || true; } \
    | tr -d '\r' \
    | awk -F': *' 'tolower($1)=="content-length"{v=$2+0} END{print v+0}')

  if [[ -z "$len" || "$len" == "0" ]]; then
    len=$( { curl -sLk --max-time 5 -r 0-0 -D - -o /dev/null "$final" 2>/dev/null || true; } \
      | tr -d '\r' \
      | awk -F'[ /]' 'tolower($1)=="content-range:"{v=$NF+0} tolower($1)=="content-length:"{cl=$2+0} END{print (v?v:cl)+0}')
  fi

  size=$(human_size "$len")

  if [[ "$final" == *"llms-full.txt"* ]]; then
    kind="full"
  else
    kind="index"
  fi

  printf "%s\t%s\t%s\n" "$kind" "$final" "$size"
  found=1
done

if [[ "$found" -eq 0 ]]; then
  echo "No llms.txt found at https://${host}" >&2
  exit 1
fi
