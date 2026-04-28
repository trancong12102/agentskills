# Agent Skills

Reusable skills and agents for AI coding agents, primarily Claude Code.

## Why ora

Most Claude Code agent frameworks (11–24 agents, 9+ hooks) add complexity to compensate for weaker models. With Opus 4.7, that complexity burns tokens without improving output. ora ships exactly two research agents:

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
<subagent_routing>
For codebase exploration use ora:Ariadne. For external research (docs, GitHub repos, library APIs) use ora:Clio. Do not call built-in Explore or general-purpose unless no ora agent fits.
</subagent_routing>

<file_search_tools>
For code search inside git-indexed dirs use fff, not shell tools (`rg`/`grep`/`ugrep` for content, `fd`/`find`/`bfs` for files):

- File lookup → `mcp__plugin_ora_fff__find_files`
- Content search → `mcp__plugin_ora_fff__grep`
- 2+ patterns in one call → `mcp__plugin_ora_fff__multi_grep`

Why: frecency-ranked, dirty-file boosted, faster on large repos.
Shell tools OK only for: non-git paths (/tmp, ~/.claude, /private), system inspection, log parsing, piped filtering of command output. Whichever shell tool is installed works — no preference among `rg`/`grep`/`ugrep` or `fd`/`find`/`bfs`.
</file_search_tools>

<investigate_before_claiming>
Read code before making claims about it. User mentions a specific file → read it first, do not speculate from memory.

For external libraries with non-trivial API surface → spawn ora:Clio rather than relying on training data; library APIs change.
</investigate_before_claiming>

<plan_before_implementing>
Use EnterPlanMode when ANY of:

- Task touches >3 files
- Data migration, API contract, or auth changes
- Acceptance criteria not stated by user
- Cross-subsystem changes

Skip plan mode for: single-file fix with obvious site, mechanical rename, config/typo fix, clearly-scoped refactor.
</plan_before_implementing>

<think_before_acting>

- Multiple reasonable interpretations of the task → state them, do not pick silently
- Simpler approach exists than asked → say so, push back
- Unclear enough to block correct execution → stop and ask the user (via AskUserQuestion); do not guess
- Translate vague goals into verifiable success criterion ("add validation" → "tests for invalid inputs pass")
- Mark task complete only after running actual check; do not trust own summary
  </think_before_acting>

<ask_user_question_tool>
When asking the user, use AskUserQuestion tool (not plain text). Load once per session via `ToolSearch` with `select:AskUserQuestion` — tool stays available after.
</ask_user_question_tool>
```

`think_before_acting` is adapted from [forrestchang/andrej-karpathy-skills](https://github.com/forrestchang/andrej-karpathy-skills).

## License

[MIT](./LICENSE) — Cong Tran
