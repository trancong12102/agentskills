# Agent Skills

Reusable skills and agents for AI coding agents, primarily Claude Code.

## Why ora

Most Claude Code agent frameworks (11–24 agents, 9+ hooks) add complexity to compensate for weaker models. With Opus 4.7, that complexity burns tokens without improving output. ora ships exactly two research agents:

- **Ariadne** — codebase exploration (semantic + keyword search across local files)
- **Clio** — external research (docs, web, GitHub repos)

Both isolate search context from the main conversation — broad queries never pollute your main window. The plugin has no hooks and no planning/verification/execution agents. Planning and verification happen inline in the main agent, shaped by behavioral rules in your `CLAUDE.md` (see Configuration below).

## Getting Started

### Prerequisites

ora ships two MCP servers — both binaries must exist on `PATH` before installing the plugin:

- [`fff-mcp`](https://github.com/dmtrKovalenko/fff.nvim) — fast file finder, frecency-ranked.
- [`ccc`](https://github.com/cocoindex-io/cocoindex-code) — semantic code search (cocoindex-code). See the repo for install and configuration.

Install `fff-mcp`:

```shell
# Install the prebuilt binary to ~/.local/bin/fff-mcp
curl -L https://dmtrkovalenko.dev/install-fff-mcp.sh | bash

# Ensure ~/.local/bin is on PATH (zsh shown — adjust for bash/fish)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc

# Verify
which fff-mcp   # should print ~/.local/bin/fff-mcp
```

The binary is statically linked (musl on Linux); no Node, Rust toolchain, or runtime dependency is required.

### Install

```shell
# Plugin (agents)
/plugin marketplace add trancong12102/agentskills
/plugin install ora@agentskills

# Skills (optional, standalone)
bunx skills add trancong12102/agentskills -g -y -a claude-code

# Other plugins
/plugin install sound-notify@agentskills
```

### Credentials

| Skill      | Credential     | How to get                                                                                           |
| ---------- | -------------- | ---------------------------------------------------------------------------------------------------- |
| `godfetch` | `ctx7 login`   | One-time login via `bunx ctx7@latest login` (library docs from [context7.com](https://context7.com)) |
| `oracle`   | Codex CLI auth | Run `codex login` after installing [Codex CLI](https://github.com/openai/codex)                      |

<details>
<summary>Codex CLI setup for oracle / council-review</summary>

Add to `~/.codex/config.toml`:

```toml
[profiles.oracle]
model = "gpt-5.5"
model_reasoning_effort = "xhigh"
approval_policy = "never"
sandbox_mode = "read-only"
```

</details>

## ora Plugin

| Agent         | Model  | Role                                                                           |
| ------------- | ------ | ------------------------------------------------------------------------------ |
| `ora:Ariadne` | Sonnet | Codebase exploration — traces flows, finds implementations, maps architecture. |
| `ora:Clio`    | Sonnet | External research — fetches docs, searches GitHub repos, checks versions.      |

## Configuration

ora is just two agents — workflow behavior lives in your `~/.claude/CLAUDE.md`. Recommended setup:

```markdown
## Subagent routing

- **Codebase exploration** → ora:Ariadne, not built-in Explore.
- **External research** (docs, GitHub repos, library APIs) → ora:Clio, not main-agent curl/gh/WebFetch. Even a single independent lookup goes to Clio. Multiple independent lookups in the same turn → spawn parallel Clio agents in one message. "Independent" = query does not depend on a prior tool call's output in this turn. Why: main-agent curl|grep dumps raw HTML/grep noise into context; Clio returns synthesized answers and parallelizes trivially. Skip Clio for empirical tests against actual endpoints, iterative drilldown (step N+1 needs step N's output), and system inspection (dig/brew/ps).

Fall back to built-in only if both ora agents unavailable or task falls outside codebase and external-research categories.

## File search tools

For code search inside git-indexed dirs use ccc or fff, not shell tools. Route by what you have in hand:

- **Only a concept / feature / question, no symbol to type** → `mcp__plugin_ora_ccc__search`. Triggers: "how does X work", "where do we handle Y", a feature described in prose, or you are about to enumerate guess-words for one feature. Why: ccc ranks by meaning, so one query catches synonyms a grep would miss.
- **Exact identifier you can spell** (function / class / variable / constant name) → `mcp__plugin_ora_fff__grep`. Why: faster and exhaustive vs ccc's top-K-by-score; do not pay the semantic-search cost for a known token.
- **Naming variants of ONE identifier** (snake_case + PascalCase, definition + alias) → `mcp__plugin_ora_fff__multi_grep`. Scope: variants of one symbol like `['ActorAuth', 'PopulatedActorAuth', 'actor_auth']` — not for enumerating a feature's vocabulary, that is ccc.
- **File by name** → `mcp__plugin_ora_fff__find_files`. Why: frecency-ranked, dirty-file boosted.

Shotgun-grep anti-pattern: about to write an OR-pattern enumerating guesses for one feature — `grep "FreeGift|ProgressBar|GiftModal|percentOff|salepify"`, `grep "modal\.com|import modal|@app\.function|@stub"`, `multi_grep(['safeAreaInset','tabViewBottomAccessory','tabBarMinimizeBehavior','toolbar(.hidden'])` — use ccc instead. Why: a 5-term OR-pattern is fragile and noisy; ccc with one prose query catches synonyms by meaning. Trigger check: ≥3 guess-words for the same feature → switch to ccc.

Shell `grep`/`find` are OK only for non-git paths, system inspection, log parsing, and piped filtering of command output.

## Before acting

- **Investigate local code first** — read code before claiming. User mentions a specific file → read it, do not speculate from memory. Why: training data does not reflect this codebase; speculation produces confidently wrong answers.
- **Look up libraries, frameworks, tools first** — these (plus package versions, framework patterns / best practices, cloud APIs, deprecation status) change frequently; training data goes stale. Do not answer without loading the matching skill or spawning ora:Clio first — trigger on the topic, not on self-judgment of "do I know this". Cost is near zero; stale knowledge breaks implementations. Skip only when task is clearly outside these categories (pure language syntax, math, project-internal code).
- **Plan when scope is non-trivial** — do not implement without calling EnterPlanMode tool first when task touches >3 files, involves data migration / API contract / auth changes, acceptance criteria not stated, or crosses subsystems. Skip for single-file fix with obvious site, mechanical rename, config/typo fix, refactor fully contained in one file.
- **State ambiguity before acting** — multiple reasonable interpretations → list them and ask. Do not pick silently.
- **Push back on scope** — simpler approach exists than asked → say so before implementing.
- **Ask when blocked** — task unclear enough to block correct execution → ask via AskUserQuestion tool, do not guess.
- **Mark complete only after verification** — translate vague goals into a verifiable criterion first ("add validation" → "tests for invalid inputs pass"). Do not mark done from own summary — run the actual check.
```

The decision-discipline bullets under "Before acting" (State ambiguity, Push back on scope, Ask when blocked, Mark complete only after verification) are adapted from [forrestchang/andrej-karpathy-skills](https://github.com/forrestchang/andrej-karpathy-skills).

## License

[MIT](./LICENSE) — Cong Tran
