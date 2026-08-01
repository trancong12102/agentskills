---
name: repo-research
description: "Researches source code, issues, PRs, releases, and git history in external repositories on GitHub, GitLab, or Bitbucket. Use when exploring how a library implements something, searching code across public repos, reading files from a known repo, or checking issues, PRs, and release info. Do not use for local codebase search or library documentation lookups."
---

# repo-research

Defaults by intent:

- Concept question in one GitHub repo ("how does lib X do Y") → `mcp__plugin_ora_morph__github_codebase_search`. Citations are real; the synthesis can misread the code — verify by reading cited files before relaying load-bearing conclusions.
- Exact-identifier search across repos, GitLab/Bitbucket targets, symbol navigation, git history → Sourcegraph MCP (`keyword_search`, `nls_search`, `read_file`, `go_to_definition`, `find_references`, `commit_search`, `diff_search`).
- Deep dive (3+ files, branching follow-ups, ast-grep) → `bash scripts/git-clone.sh <repo>` — shallow-clones into a `~/.cache/clio-repos/` cache and echoes the path. Accepts `owner/repo`, HTTPS, or SSH; `--refresh` for the latest commit.
- Issues / PRs / releases / single known file → `gh` CLI: `gh issue view`, `gh pr view`, `gh release view|list`, `gh api repos/.../contents/<path>`.

Don't WebFetch github.com or raw.githubusercontent.com — `gh` returns the same content structured. The releases page is the sharpest trap: its relative timestamps ("2 weeks ago") get hallucinated into training-era dates.

Sourcegraph specifics:

- `keyword_search` is AND + literal — one wrong term zeroes results; use for concrete identifiers. `nls_search` is OR + stemming, not semantic — pass 2–5 extracted keywords, not sentences.
- Anchor repo filters: `repo:^github\.com/foo/bar$` — unanchored `repo:foo/bar` matches forks too.
- Search results annotated `repo@revision`: pass `repo` and `revision` separately to `read_file`, or you silently read default-branch HEAD.
- Code newer than ~24 h isn't indexed; unindexed repo or 401/403 → fall back to git-clone.
