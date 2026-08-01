#!/usr/bin/env bash
# Ensure a public git repo is shallow-cloned into a local cache and echo its
# absolute path. Works with any git host (GitHub, GitLab, Bitbucket, self-hosted).
# Idempotent — safe to call repeatedly.
#
# Usage:
#   git-clone.sh <repo> [--branch BRANCH] [--refresh] [--cache-dir DIR]
#
# Repo argument forms:
#   owner/repo                       GitHub shortcut → https://github.com/owner/repo.git
#   https://host/path[.git]          Any HTTPS git URL
#   git@host:path[.git]              SSH form
#   ssh://git@host[:port]/path[.git] Explicit SSH URL
#
# Options:
#   --branch X       Clone a specific branch (default: repo default branch)
#   --refresh        Pull latest if cache exists (default: keep cached state)
#   --cache-dir DIR  Override cache root (default: ~/.cache/clio-repos)
#
# Output: absolute path to the cached repo directory on stdout.
#
# Examples:
#   git-clone.sh vercel/next.js
#   git-clone.sh https://gitlab.com/group/subgroup/proj
#   git-clone.sh git@github.com:vercel/next.js.git --branch canary
#   git-clone.sh https://gitlab.jmango360.com/team/repo --refresh

set -euo pipefail

repo=""
branch=""
refresh=0
cache_base="${HOME}/.cache/clio-repos"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --branch)
      [[ $# -lt 2 ]] && { echo "Error: --branch requires a value" >&2; exit 1; }
      branch="$2"; shift 2 ;;
    --refresh)
      refresh=1; shift ;;
    --cache-dir)
      [[ $# -lt 2 ]] && { echo "Error: --cache-dir requires a value" >&2; exit 1; }
      cache_base="$2"; shift 2 ;;
    -h|--help)
      sed -n '2,/^$/p' "$0" | sed 's/^# \{0,1\}//'
      exit 0 ;;
    -*)
      echo "Error: unknown flag '$1'" >&2; exit 1 ;;
    *)
      [[ -n "$repo" ]] && { echo "Error: unexpected positional arg '$1'" >&2; exit 1; }
      repo="$1"; shift ;;
  esac
done

if [[ -z "$repo" ]]; then
  echo "Error: missing repo argument" >&2
  echo "Usage: git-clone.sh <repo> [--branch X] [--refresh] [--cache-dir DIR]" >&2
  exit 1
fi

# Normalize repo argument into a clone URL.
if [[ "$repo" == *"://"* ]]; then
  url="$repo"
elif [[ "$repo" == *"@"*":"* ]]; then
  # SSH form: git@host:path
  url="$repo"
elif [[ "$repo" == */* && "$repo" != *" "* ]]; then
  # GitHub shortcut: owner/repo
  url="https://github.com/${repo}.git"
else
  echo "Error: cannot parse repo argument '$repo' — use owner/repo, https://..., or git@host:path" >&2
  exit 1
fi

# Derive a stable cache key from the URL.
key="$url"
key="${key#http://}"
key="${key#https://}"
key="${key#ssh://}"
key="${key#git@}"
# SSH form host:path → host/path (replace first ':' with '/')
key="${key/:/\/}"
# Strip trailing .git
key="${key%.git}"
# Strip trailing /
key="${key%/}"
# Replace path separators with --
key="${key//\//--}"
[[ -n "$branch" ]] && key="${key}--${branch//\//_}"

dest="${cache_base}/${key}"

mkdir -p "$cache_base"

if [[ -d "${dest}/.git" ]]; then
  if [[ "$refresh" -eq 1 ]]; then
    git -C "$dest" fetch --depth=1 --quiet origin "${branch:-HEAD}" >&2
    git -C "$dest" reset --hard --quiet "FETCH_HEAD" >&2
  fi
else
  clone_args=(--depth=1 --quiet)
  [[ -n "$branch" ]] && clone_args+=(--branch "$branch" --single-branch)
  git clone "${clone_args[@]}" "$url" "$dest" >&2
fi

echo "$dest"
