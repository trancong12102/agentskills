# Agent Skills

Reusable skills and agents for AI coding agents, primarily Claude Code.

## Why ora

Most Claude Code agent frameworks (11–24 agents, 9+ hooks) add complexity to compensate for weaker models. With Opus 4.8, that complexity burns tokens without improving output. ora ships exactly two research agents:

- **Ariadne** — codebase exploration (semantic search, keyword search, and file discovery across local files)
- **Clio** — external research (docs, web, GitHub repos)

Both isolate search context from the main conversation — broad queries never pollute your main window. The plugin has no hooks and no planning/verification/execution agents. Planning and verification happen inline in the main agent, shaped by behavioral rules in your `CLAUDE.md` (see Configuration below).

## Getting Started

### Prerequisites

ora ships two MCP servers, each with its own setup:

**[`fff-mcp`](https://github.com/dmtrKovalenko/fff.nvim)** — stdio MCP, fast file finder (frecency-ranked). Install the binary:

```shell
# Install the prebuilt binary to ~/.local/bin/fff-mcp
curl -L https://dmtrkovalenko.dev/install-fff-mcp.sh | bash

# Ensure ~/.local/bin is on PATH (zsh shown — adjust for bash/fish)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc

# Verify
which fff-mcp   # should print ~/.local/bin/fff-mcp
```

Statically linked (musl on Linux) — no Node, Rust toolchain, or runtime dependency.

**[`sourcegraph`](https://sourcegraph.com/docs/api/mcp)** — HTTP MCP, cross-repo code search across 2M+ OSS repos. See [Credentials](#credentials) for token setup.

**[`morph`](https://docs.morphllm.com/mcpquickstart)** — stdio MCP via `bunx @morphllm/morphmcp@latest`, semantic codebase search (`codebase_search` for local, `github_codebase_search` for GitHub deps). `edit_file` is disabled by default. See [Credentials](#credentials) for API key setup.

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

| Skill / Plugin | Credential          | How to get                                                                                                                                                                          |
| -------------- | ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `godfetch`     | `ctx7 login`        | One-time login via `bunx ctx7@latest login` (library docs from [context7.com](https://context7.com))                                                                                |
| `oracle`       | Codex CLI auth      | Run `codex login` after installing [Codex CLI](https://github.com/openai/codex)                                                                                                     |
| `ora` (MCP)    | `SOURCEGRAPH_TOKEN` | Generate a PAT at [sourcegraph.com/user/settings/tokens/new](https://sourcegraph.com/user/settings/tokens/new) (scope `mcp`, no expiration), then `export SOURCEGRAPH_TOKEN=sgp_…`. |
| `ora` (MCP)    | `MORPH_API_KEY`     | Sign up at [morphllm.com](https://morphllm.com), generate an API key, then `export MORPH_API_KEY=…`.                                                                                |

> Both tokens are read from your shell at startup — if either is unset, Claude Code fails to parse `.mcp.json`. Export both, or remove the corresponding server block. (OAuth via `/mcp` is an alternative for Sourcegraph but expires quickly.)

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
## Search and delegation

- Keyword / exact-file search → `fff` MCP tools. Semantic / concept search → `mcp__plugin_ora_morph__codebase_search` or `ora:Ariadne`.
- Do not use built-in Explore or generic Agent when `ora:Ariadne` (codebase) or `ora:Clio` (external research) can do the job.
- Delegate to `ora:Ariadne` / `ora:Clio` when the exploration is isolated from current work and you just need the answer back. Keep search in main agent when iteration / reasoning needs the trail (debug, trace, ongoing implementation).

## Subagent model selection

Pass `model` to Agent tool by task complexity (not by agent identity):

- **opus** — deep analysis, architecture mapping, elusive debug, security review, cross-module trace
- **sonnet** — search / lookup, doc fetch, code review, codegen, format

Omit `model` → inherits parent. Why: same agent (e.g. Ariadne) serves both "find file X" (sonnet) and "map request lifecycle" (opus); routing belongs at call site, not in agent definition.
```

## License

[MIT](./LICENSE) — Cong Tran
