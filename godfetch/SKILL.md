---
name: godfetch
description: "Unified external research for documentation, GitHub code, and package versions. Use when the user needs to look up library docs or API references, search code in a public GitHub repo, check the latest version of a package, verify whether a dependency is deprecated, or any task requiring external knowledge beyond the local codebase and training data."
---

# godfetch

Unified external research — look up library documentation, search source code in any git repository, and check package versions from a single skill.

## Routing

| Intent                          | Primary tool                       | Fallback                              |
| ------------------------------- | ---------------------------------- | ------------------------------------- |
| Library docs, API reference     | `context7`                         | WebSearch for niche/undocumented libs |
| Changelogs, breaking changes    | `context7`                         | `gh api contents` for CHANGELOG.md    |
| Code in public git repos        | `git-clone` + shell tools          | `gh search code` for exact matches    |
| GitHub issues                   | `gh issue view <N>`                | `gh search issues` for discovery      |
| GitHub PRs                      | `gh pr view <N>`                   | `gh search prs` for discovery         |
| Single GitHub file (known path) | `gh api repos/.../contents/<path>` | `git-clone` + shell tools             |
| Package version, deprecation    | `deps-dev`                         | `npm view` for npm-only metadata      |
| npm package info (non-version)  | `npm view <pkg>` (Bash)            | WebSearch for community sentiment     |
| General web lookup              | WebSearch → WebFetch               | —                                     |
| Comparison / decision           | `context7` + WebSearch             | `git-clone` + shell tools for usage   |

For mixed requests, launch all relevant tools in parallel. Cloning is I/O-bound — start `git-clone` in the background and run `context7`/WebSearch concurrently to mask clone latency.

### GitHub access rules

Do not use WebFetch on github.com or raw.githubusercontent.com URLs — use the right tool:

| GitHub content            | Use                                   | Never                                |
| ------------------------- | ------------------------------------- | ------------------------------------ |
| Source code (exploration) | `git-clone` + shell tools             | browsing files via `gh api contents` |
| Source file (known path)  | `gh api repos/.../contents/<path>`    | `WebFetch` raw.githubusercontent.com |
| Issues                    | `gh issue view <N> --repo owner/repo` | `WebFetch` github.com/.../issues/N   |
| Pull requests             | `gh pr view <N> --repo owner/repo`    | `WebFetch` github.com/.../pull/N     |
| Issue/PR search           | `gh search issues "q" --repo ...`     | `WebFetch` github.com/issues?q=...   |
| CHANGELOG.md              | `context7` or `gh api contents`       | `WebFetch` blob/ or raw URLs         |

### Search discipline

- **deps-dev for versions**: when checking latest version, deprecation, or comparing installed vs latest — always use `deps-dev` first. Only fall back to `npm view` or WebSearch if deps-dev errors or the package is private.
- **Context7 first**: for any library with >1K GitHub stars, try `context7` before WebSearch.

## context7 — Library Documentation

Two-step workflow via the official `ctx7` CLI. Requires `ctx7 login` once (no API key env var).

### Step 1: Resolve library ID

```bash
bunx ctx7@latest library <name> [query]
```

Lists library candidates with their Context7 IDs (e.g. `/websites/react_dev`), trust scores, and snippet counts. The optional `[query]` re-ranks results by relevance — pass it whenever you already know the topic. Add `--json` for machine-readable output.

### Step 2: Fetch documentation

```bash
bunx ctx7@latest docs <libraryId> "<query>"
```

Returns markdown snippets ranked by relevance. Add `--json` for structured output. If the first answer is shallow or off-topic, retry with `--research` — it spins up sandboxed agents that read the source repo and run a live web search, at higher cost.

**Rules:**

- One-time setup: `bunx ctx7@latest login` (interactive). Verify with `bunx ctx7@latest whoami`.
- Always resolve the library ID first — IDs are not guessable.
- Write specific queries — `"useState hook with objects"` beats `"hooks"`. The query drives relevance ranking on both commands.
- Use `--research` only as a retry when the default answer was insufficient, not by default — it's slower and more expensive.

Reference: `references/context7.md`

## git-clone — Source Code Exploration

Shallow-clone any public git repo into a local cache and explore the working tree with shell tools. Cache lives at `~/.cache/clio-repos/` and is reused across sessions.

```bash
bash scripts/git-clone.sh <repo> [--branch X] [--refresh]
```

The script echoes the absolute path of the cached clone. Subsequent calls for the same repo return the cached path instantly (no re-clone).

**Repo argument forms:**

- `owner/repo` — GitHub shortcut (e.g. `vercel/next.js`)
- Full HTTPS URL — works for any host (`https://gitlab.com/...`, `https://gitlab.jmango360.com/...`)
- SSH form — `git@host:path` (requires SSH key configured)

**Parallelization:** clone is I/O-bound (1-3s for small/medium repos). When researching a topic that needs both docs and source code, dispatch the clone and `context7`/WebSearch in parallel — by the time docs return, the clone is ready to explore.

**Rules:**

- Do not read script source code. Run with `--help` for usage.
- Default cache is `~/.cache/clio-repos/`; override with `--cache-dir` when needed.
- Caches are reused — pass `--refresh` only when you need the latest commit.
- For one-off file fetches by exact path, prefer `gh api repos/.../contents/<path>` — no clone overhead.

## deps-dev — Package Versions

Query latest stable versions from public registries. No API key needed.

```bash
python3 scripts/get-versions.py <system> <pkg1> [pkg2] ...
```

**Supported ecosystems:**

| Ecosystem | System ID  | Example                           |
| --------- | ---------- | --------------------------------- |
| npm       | `npm`      | `express`, `@types/node`          |
| PyPI      | `pypi`     | `requests`, `django`              |
| Go        | `go`       | `github.com/gin-gonic/gin`        |
| Cargo     | `cargo`    | `serde`, `tokio`                  |
| Maven     | `maven`    | `org.springframework:spring-core` |
| NuGet     | `nuget`    | `Newtonsoft.Json`                 |
| RubyGems  | `rubygems` | `rails`, `sidekiq`                |

Output: TSV with columns `package`, `version`, `published`, `status`.

**Rules:**

- Do not read script source code. Run directly or use `--help`.
- Batch lookups when possible — pass multiple package names in one call.
- Flag deprecated packages — if status says `deprecated`, suggest an alternative.

Reference: `references/deps-dev.md`
