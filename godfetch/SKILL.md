---
name: godfetch
description: "Unified external research for documentation, GitHub code, and package versions. Use when the user needs to look up library docs or API references, search code in a public GitHub repo, check the latest version of a package, verify whether a dependency is deprecated, or any task requiring external knowledge beyond the local codebase and training data."
---

# godfetch

Unified external research — look up library documentation, search source code in any git repository, and check package versions from a single skill.

## Routing

| Intent                                   | Primary tool                              | Fallback                                         |
| ---------------------------------------- | ----------------------------------------- | ------------------------------------------------ |
| Library docs, API reference              | `llms-probe` → `WebFetch` llms.txt        | `context7` if no llms.txt published              |
| Changelogs, breaking changes             | `llms-probe` → `WebFetch` llms.txt        | `gh api contents` for CHANGELOG.md               |
| Code in public git repos                 | `git-clone` + shell tools                 | `gh search code` for exact matches               |
| GitHub issues                            | `gh issue view <N>`                       | `gh search issues` for discovery                 |
| GitHub PRs                               | `gh pr view <N>`                          | `gh search prs` for discovery                    |
| GitHub releases (versions, dates, notes) | `gh release view <tag> --repo owner/repo` | `gh release list --repo owner/repo` for browsing |
| Single GitHub file (known path)          | `gh api repos/.../contents/<path>`        | `git-clone` + shell tools                        |
| Package version, deprecation             | `deps-dev`                                | `npm view` for npm-only metadata                 |
| npm package info (non-version)           | `npm view <pkg>` (Bash)                   | WebSearch for community sentiment                |
| General web lookup                       | WebSearch → WebFetch                      | —                                                |
| Comparison / decision                    | `llms-probe` per lib + WebSearch          | `context7` for additional snippets               |

For mixed requests, launch all relevant tools in parallel. Probe and clone are I/O-bound — start them in the background and run `WebFetch`/`context7`/WebSearch concurrently to mask latency.

### GitHub access rules

Do not use WebFetch on github.com or raw.githubusercontent.com URLs — use the right tool:

| GitHub content                    | Use                                                                     | Never                                                                                                         |
| --------------------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| Source code (exploration)         | `git-clone` + shell tools                                               | browsing files via `gh api contents`                                                                          |
| Source file (known path)          | `gh api repos/.../contents/<path>`                                      | `WebFetch` raw.githubusercontent.com                                                                          |
| Issues                            | `gh issue view <N> --repo owner/repo`                                   | `WebFetch` github.com/.../issues/N                                                                            |
| Pull requests                     | `gh pr view <N> --repo owner/repo`                                      | `WebFetch` github.com/.../pull/N                                                                              |
| Issue/PR search                   | `gh search issues "q" --repo ...`                                       | `WebFetch` github.com/issues?q=...                                                                            |
| Releases (versions, dates, notes) | `gh release view/list --repo owner/repo` or `gh api repos/.../releases` | `WebFetch` github.com/.../releases — relative timestamps on the HTML get hallucinated into training-era years |
| CHANGELOG.md                      | `gh api repos/.../contents/CHANGELOG.md`                                | `WebFetch` blob/ or raw URLs                                                                                  |

### Search discipline

- **deps-dev for versions**: when checking latest version, deprecation, or comparing installed vs latest — always use `deps-dev` first. Only fall back to `npm view` or WebSearch if deps-dev errors or the package is private.
- **llms.txt first, context7 fallback**: for library docs, run `scripts/llms-probe.sh` against the docs domain before reaching for `context7`. Author-published llms.txt has no community-curation lag and no enrichment layer that can hallucinate. Fall back to `context7` only when probe returns nothing.

## llms.txt — Author-Canonical Library Documentation

Many doc sites publish [llms.txt](https://llmstxt.org/) (Markdown index of doc pages) and `llms-full.txt` (concatenated full content). These are author-published — no enrichment layer, no community-curation lag — so they reflect the deployed docs version exactly. Prefer them over `context7` when available.

### Step 1: Probe for availability

```bash
bash scripts/llms-probe.sh <docs-domain>
```

Outputs TSV `kind \t url \t size` for any found files. Probes root + common nested paths (`/docs/`, `/en/`), follows redirects, dedupes. Returns non-zero exit if nothing found.

| `kind` | Meaning                                           |
| ------ | ------------------------------------------------- |
| index  | `llms.txt` — Markdown list of doc page URLs       |
| full   | `llms-full.txt` — entire docs corpus concatenated |

Size shows `?` when the CDN strips both `Content-Length` and `Content-Range` headers (Vercel does this on react.dev) — treat `?` as unknown and prefer the index path.

### Step 2: Fetch based on what's there

| Found                                 | Action                                                                |
| ------------------------------------- | --------------------------------------------------------------------- |
| `llms-full.txt` ≤ ~500 KB             | `WebFetch` it directly — single round trip, full corpus               |
| `llms-full.txt` > ~500 KB or size `?` | `WebFetch` `llms.txt` first, pick relevant section links, fetch those |
| Only index (no full)                  | `WebFetch` the index, then fetch individual page links                |
| Probe failed                          | Fall back to `context7` (next section)                                |

For multi-section pulls, dispatch the page `WebFetch` calls in parallel.

### Known publishers

Confirmed live (April 2026): React (`react.dev`), Next.js (`nextjs.org`, content under `/docs/`), Vercel, Anthropic (`docs.anthropic.com` → `platform.claude.com`), Cloudflare (`docs.cloudflare.com` → `developers.cloudflare.com`), Supabase, Drizzle (`orm.drizzle.team`), Hono (`hono.dev`), Zod (`zod.dev`), Expo (`docs.expo.dev`), tRPC (`trpc.io`), shadcn/ui (`ui.shadcn.com`). Most Mintlify- and GitBook-hosted docs auto-publish.

Tailwind, most pre-1.0 libraries, and many community packages do not publish — those go straight to `context7`.

### Rules

- **Probe before assuming.** Adoption is uneven and paths vary (root vs `/docs/` vs redirects). Always run `llms-probe.sh` and act on the TSV — never hardcode URLs.
- **Watch file size before fetching full.** Cloudflare's `llms-full.txt` is ~46 MB and Supabase reports `?` (chunked). A blind fetch of either blows the context window. The 500 KB threshold is a heuristic — adjust to remaining context budget.
- **Index → page chain for big corpora.** Treat `llms.txt` as a routing table: parse section headings, fetch only the page URLs that match the question.

## context7 — Library Documentation (Fallback)

Reach for `context7` when `llms-probe.sh` returns nothing — the library doesn't publish llms.txt, or its docs domain isn't reachable. Coverage spans ~33K libraries via community-curated indexes; tradeoff is an enrichment layer that can introduce inaccuracies the author-published llms.txt avoids.

Two-step workflow via the official `ctx7` CLI. Requires `bunx ctx7@latest login` once (no API key env var).

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

**Parallelization:** clone is I/O-bound (1-3s for small/medium repos). When researching a topic that needs both docs and source code, dispatch the clone and `llms-probe`/`context7`/WebSearch in parallel — by the time docs return, the clone is ready to explore.

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
