---
name: repo-research
description: "Routes research in external repositories — source code, issues, PRs, releases, git history — across GitHub, GitLab, and Bitbucket. Use when exploring how a library implements something, searching code across public repos, or checking issues, PRs, and release info. Do not use for local codebase search or library documentation lookups."
---

# repo-research

Defaults by intent:

- Concept question in one GitHub repo ("how does lib X do Y") → `mcp__plugin_ora_morph__github_codebase_search`. Citations are real; the synthesis can misread the code — verify by reading cited files before relaying load-bearing conclusions.
- Exact-identifier search across repos, GitLab/Bitbucket targets, git history → Sourcegraph MCP (`keyword_search`, `nls_search`, `read_file`, `list_files`, `commit_search`, `diff_search`). For symbol navigation, `keyword_search` the identifier, then `read_file` the hits.
- Reading a known file, or a deep dive across 3+ files → `mcp__plugin_ora_ora__repo_fetch` (`clone: true` for the deep dive).
- Issues / PRs / releases → `gh` CLI: `gh issue view`, `gh pr view`, `gh release view|list`.

The releases page is the sharpest trap for a web fetch tool: its relative timestamps ("2 weeks ago") get hallucinated into training-era dates. Use `gh release` instead.

Sourcegraph specifics:

- `keyword_search` is AND + literal — one wrong term zeroes results; use for concrete identifiers. `nls_search` is OR + stemming, not semantic — pass 2–5 extracted keywords, not sentences.
- Anchor repo filters: `repo:^github\.com/foo/bar$` — unanchored `repo:foo/bar` matches forks too.
- Search results annotated `repo@revision`: pass `repo` and `revision` separately to `read_file`, or you silently read default-branch HEAD.
- Code newer than ~24 h isn't indexed; unindexed repo or 401/403 → fall back to `repo_fetch` with `clone: true`.
