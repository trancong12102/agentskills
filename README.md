# Agent Skills

A collection of reusable skills for AI coding agents, mainly for Claude Code.

## Prerequisites

Some skills require API keys or external CLI authentication. Set them up before use:

| Skill                    | Credential                 | How to get                                                                      |
| ------------------------ | -------------------------- | ------------------------------------------------------------------------------- |
| `context7`               | `CONTEXT7_API_KEY` env var | Sign up at [context7.com](https://context7.com)                                 |
| `codebase-search`        | `MORPH_API_KEY` env var    | Sign up at [morphllm.com](https://morphllm.com)                                 |
| `github-codebase-search` | `MORPH_API_KEY` env var    | Same as above                                                                   |
| `oracle`                 | Codex CLI auth             | Run `codex login` after installing [Codex CLI](https://github.com/openai/codex) |
| `council-review`         | Codex CLI auth             | Same as above                                                                   |

### Codex CLI setup

The `oracle` and `council-review` skills require [Codex CLI](https://github.com/openai/codex) with an `oracle` profile. Add this to `~/.codex/config.toml`:

```toml
[profiles.oracle]
model = "gpt-5.4"
model_reasoning_effort = "high"
approval_policy = "never"
sandbox_mode = "read-only"
```

## Installation

### Skills

```bash
# All skills
bunx skills add trancong12102/agentskills -g -y -a claude-code

# Or individual skills
bunx skills add trancong12102/agentskills -g -y -a claude-code -s context7
bunx skills add trancong12102/agentskills -g -y -a claude-code -s council-review
bunx skills add trancong12102/agentskills -g -y -a claude-code -s deps-dev
bunx skills add trancong12102/agentskills -g -y -a claude-code -s oracle
```

### Plugins

| Plugin                                 | Description                                                                                    |
| -------------------------------------- | ---------------------------------------------------------------------------------------------- |
| [ora](./plugins/ora)                   | 6 specialized subagents for exploration, planning, and execution (see [Agents](#agents) below) |
| [sound-notify](./plugins/sound-notify) | Play macOS notification sounds when Claude stops or asks a question                            |

```shell
/plugin marketplace add trancong12102/agentskills
/plugin install ora@agentskills
/plugin install sound-notify@agentskills

# Enable auto-update
/plugin marketplace update agentskills
```

## Agents

The `ora` plugin ships 6 specialized subagents. Four are hook-enforced (automatically triggered at the right time), two are spawn-on-demand.

### On-Demand Agents

| Agent         | Model  | Description                                                                       |
| ------------- | ------ | --------------------------------------------------------------------------------- |
| `ora:Ariadne` | Sonnet | Codebase exploration — traces flows, finds implementations, maps architecture.    |
| `ora:Clio`    | Sonnet | External research — fetches docs, searches GitHub repos, looks up best practices. |

### Hook-Enforced Agents

| Agent            | Model  | Hook                     | Description                                                                                                                            |
| ---------------- | ------ | ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------- |
| `ora:Metis`      | Opus   | PreToolUse EnterPlanMode | Intent classification + pre-analysis. Surfaces risks, generates directives, asks clarifying questions via AskUserQuestion.             |
| `ora:Momus`      | Sonnet | PreToolUse ExitPlanMode  | Plan validation for plans with 2+ steps. Checks executability, references, blockers. Approval-biased — rejects only for true blockers. |
| `ora:Atlas`      | Opus   | PreToolUse ExitPlanMode  | Wave dispatch for plans with code tasks. Groups tasks into parallel waves, assigns agents, defines learning capture.                   |
| `ora:Hephaestus` | Opus   | Dispatched by Atlas      | Autonomous deep worker — receives a goal, works independently in a worktree, returns finished code with structured summary.            |

### Workflow

Research agents (Ariadne, Clio) are spawned on-demand throughout the workflow. Planning and execution agents are triggered automatically by hooks.

```text
User request
  │
  ▼
EnterPlanMode ────────────────────────────────────────────
  │  PreToolUse hook
  │  ├─ ora:Metis      — intent classification + directives
  │  └─ AskUserQuestion — clarify open questions from Metis
  │
  ▼
Plan mode (model writes plan) ────────────────────────────
  │  ora:Ariadne / ora:Clio spawned as needed for context
  │
  ▼
ExitPlanMode ─────────────────────────────────────────────
  │  PreToolUse hook
  │  ├─ ora:Momus      — validate plan (2+ steps)
  │  └─ ora:Atlas      — wave dispatch (code tasks)
  │
  │  User approves plan
  │
  ▼
Execution ────────────────────────────────────────────────
     ora:Hephaestus — dispatched by Atlas per code task
     ora:Ariadne    — dispatched by Atlas for exploration
     ora:Clio       — dispatched by Atlas for research
```

## System Prompt Enhancement

Skills and ora agents work best when Claude Code is instructed at the **system prompt level** to use them proactively. CLAUDE.md instructions are advisory and can be ignored — system prompt instructions have the highest compliance.

### Setup

1. Create `~/.claude/system-prompt-extra.md`:

```xml
<investigate_before_responding>
Training data is from mid-2025 and is likely outdated. Before responding to any
technical task, verify with a matching skill or ora:Clio research. The cost of
loading a skill or spawning a research agent is near zero — the cost of outdated
knowledge is a broken implementation.
</investigate_before_responding>

<use_skills_proactively>
Load matching skills before writing code or giving technical advice. Skip only
when the task is clearly unrelated to any available skill (e.g., git operations,
file renaming, simple config edits).
</use_skills_proactively>

<subagent_routing>
Use specialized ora agents instead of built-in Explore or general-purpose agents
for any task ora agents can handle. Use ora:Ariadne for codebase exploration,
ora:Clio for external research and documentation lookups. Reserve Glob and Grep
for simple, targeted searches (specific file, class, or function by name).
</subagent_routing>

<plan_before_implementing>
Enter plan mode before implementing. Skip only for truly trivial tasks —
single-file edits, renaming, simple config changes, typo fixes. If a task
touches 2+ files or has any ambiguity, plan first.
</plan_before_implementing>
```

2. Add a shell alias to auto-inject on every session:

```bash
alias cc='claude --append-system-prompt-file ~/.claude/system-prompt-extra.md'
```

### Why system prompt instead of CLAUDE.md?

Claude Code wraps CLAUDE.md content in a `<system-reminder>` with the disclaimer _"this context may or may not be relevant"_. The model treats it as advisory and skips instructions when it feels confident answering from training data. `--append-system-prompt-file` injects at the system prompt level with no disclaimer wrapper — the same priority as Claude Code's own built-in instructions.

## License

[MIT](./LICENSE) — Cong Tran
