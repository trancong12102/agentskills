# Agent Skills

Reusable skills and agents for AI coding agents, primarily Claude Code.

## Why ora

Most Claude Code agent frameworks (11–24 agents, 9+ hooks) add complexity to compensate for weaker models. With Opus 4.6, that complexity burns tokens without improving output. ora ships exactly two research agents:

- **Ariadne** — codebase exploration (enhanced contextual grep over local files)
- **Clio** — external research (docs, web, GitHub repos)

Both isolate search context from the main conversation — broad queries never pollute your main window. The plugin has no hooks and no planning/verification/execution agents. Planning and verification happen inline in the main agent, shaped by behavioral rules in your `CLAUDE.md` (see Configuration below).

## Getting Started

### Prerequisites

ora bundles [`fff-mcp`](https://github.com/dmtrKovalenko/fff.nvim) (fast file finder, frecency-ranked) as an MCP server, so the binary must exist on `PATH` before installing the plugin:

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

| Skill      | Credential                 | How to get                                                                      |
| ---------- | -------------------------- | ------------------------------------------------------------------------------- |
| `godgrep`  | `MORPH_API_KEY` env var    | Sign up at [morphllm.com](https://morphllm.com) (codebase-search)               |
| `godfetch` | `CONTEXT7_API_KEY` env var | Sign up at [context7.com](https://context7.com) (library docs)                  |
| `godfetch` | `MORPH_API_KEY` env var    | Sign up at [morphllm.com](https://morphllm.com) (GitHub code search)            |
| `oracle`   | Codex CLI auth             | Run `codex login` after installing [Codex CLI](https://github.com/openai/codex) |

<details>
<summary>Codex CLI setup for oracle / council-review</summary>

Add to `~/.codex/config.toml`:

```toml
[profiles.oracle]
model = "gpt-5.4"
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
<investigate_before_responding>
Do not respond to tech task without loading matching skill or spawning ora:Clio research first. Cost near zero; stale knowledge break implementations. Skip only when task clearly unrelated to any skill.

Never speculate on code not opened. User reference specific file → read file first. Ground all claims in investigated state.
</investigate_before_responding>

<subagent_routing>
Do not use built-in Explore or general-purpose agent — use ora:Ariadne (local codebase) and ora:Clio (external sources). general-purpose escape hatch only when no ora agent fits.

Do not serialize independent subagent work. Fan out parallel: spawn multiple subagents in single turn when work covers independent items/files.

Do not respawn agents. Follow-up on agent already spawned this session → SendMessage with returned agentId. Fresh spawn lose cached context and repeat tool calls. New Agent call only when topic genuinely different.
</subagent_routing>

<file_search_tools>
Do not use built-in Grep/Glob or shell `grep`/`find`/`rg` for file search in git-indexed dir. Use fff tools shipped with ora plugin: `mcp__plugin_ora_fff__find_files` (file lookup), `mcp__plugin_ora_fff__grep` (content search), `mcp__plugin_ora_fff__multi_grep` (OR logic across patterns). Frecency-ranked, faster, dirty-file boost. Fallback to built-in only when outside git index.
</file_search_tools>

<plan_before_implementing>
Do not implement without calling EnterPlanMode tool first when task ambiguous, spans multiple subsystems, or acceptance criteria unclear. Skip plan mode for clearly-scoped changes: single-file bug fix with obvious fix site, mechanical rename/refactor, config/typo fix.
</plan_before_implementing>

<think_before_coding>
Do not start implementing without stating assumptions and flagging tradeoffs. Multiple reasonable interpretations → present them, do not pick silent. Simpler approach exist than asked → say so, push back. Unclear enough to block correct execution → stop and ask, do not guess.
</think_before_coding>

<goal_driven_execution>
Do not accept vague goals. Translate each task into verifiable success criterion before implementing ("add validation" → "write tests for invalid inputs, then make them pass"). Do not mark task complete until success criterion met — read actual code, run actual check, do not trust own summary.
</goal_driven_execution>

<ask_user_question_tool>
Do not ask user via plain response. Use AskUserQuestion tool — load via `ToolSearch` with `select:AskUserQuestion`.
</ask_user_question_tool>
```

`think_before_coding` and `goal_driven_execution` are adapted from [forrestchang/andrej-karpathy-skills](https://github.com/forrestchang/andrej-karpathy-skills).

## License

[MIT](./LICENSE) — Cong Tran
