# Agent Skills

Reusable skills and agents for AI coding agents, primarily Claude Code.

## Why ora

Most Claude Code agent frameworks (11–24 agents, 9+ hooks) add complexity to compensate for weaker models — guardrailing planning quality, enforcing step-by-step discipline, gating tool access. With Opus 4.6, that complexity burns tokens without improving output.

ora keeps only what a strong model can't do alone:

1. **Plan** — Opus plans inline. No plan-review agent needed — Opus rarely produces unexecutable plans, and when it does, the executor catches it.
2. **Execute** — Hephaestus runs in parallel worktrees. This is capability extension, not model compensation — a single agent can't work on multiple isolated branches simultaneously.
3. **Verify** — Aletheia checks acceptance criteria against actual code. Self-assessment has blind spots regardless of model strength — independent verification catches what the implementer misses.
4. **Research** — Ariadne (codebase) and Clio (external) isolate search context from the main conversation, preventing context pollution from broad queries.

Two lightweight hooks inject context reminders (load workflow skill, run verification) — no blocking, no workflow enforcement, no weaker-model reviewing stronger-model output. Session resume via SendMessage saves ~70% tokens per round-trip.

### Comparison

|                      | ora                                       | [superpowers]                | [get-shit-done]                 | [oh-my-openagent]                          |
| -------------------- | ----------------------------------------- | ---------------------------- | ------------------------------- | ------------------------------------------ |
| Architecture         | 4 agents, multi-model                     | 14 skills, single model      | 24 agents, single model         | 11 agents, multi-model                     |
| Runtime deps         | Zero                                      | Zero                         | Zero (Node >= 22)               | Heavy (ast-grep, MCP SDK, native binaries) |
| Token strategy       | Session resume (~70% savings on retries)  | On-demand skill loading      | File-based context (.planning/) | Always-on MCPs + injected                  |
| Workflow enforcement | 2 hooks (context injection)               | 1 hook (soft)                | 9 hooks (mostly advisory)       | Behavioral (intent gate, todo enforcer)    |
| Session management   | Resume via SendMessage                    | Fresh subagent per task      | File-based persistence          | Auto-recovery, no explicit resume          |
| Composability        | Skills via frontmatter, agents composable | Skills loaded via Skill tool | Edit agent markdown files       | JSONC config, prompt_append                |
| Host                 | Claude Code                               | Claude Code + 7 others       | 13 runtimes                     | OpenCode (Claude Code compat layer)        |
| License              | MIT                                       | MIT                          | MIT                             | SUL-1.0                                    |

[superpowers]: https://github.com/obra/superpowers
[get-shit-done]: https://github.com/gsd-build/get-shit-done
[oh-my-openagent]: https://github.com/code-yeongyu/oh-my-openagent

**Where ora fits**: minimal hooks (2 context-injecting reminders), no compiled plugins, no native binaries, no mandatory MCP servers — 4 markdown agents + 2 lightweight hooks. Each agent exists because it extends what a single model can't do (parallelism, context isolation, independent verification), not because the model needs guardrails.

## Getting Started

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
model_reasoning_effort = "high"
approval_policy = "never"
sandbox_mode = "read-only"
```

</details>

## ora Plugin

### Agents

| Agent            | Model  | Role                                                                           |
| ---------------- | ------ | ------------------------------------------------------------------------------ |
| `ora:Hephaestus` | Opus   | Deep worker — implements in worktrees, squash-merged by caller.                |
| `ora:Aletheia`   | Sonnet | Verification — checks acceptance criteria against actual code, not summaries.  |
| `ora:Ariadne`    | Sonnet | Codebase exploration — traces flows, finds implementations, maps architecture. |
| `ora:Clio`       | Sonnet | External research — fetches docs, searches GitHub repos, checks versions.      |

### Workflow & Hooks

The `/ora` skill contains the full workflow (plan → execute → verify → review). Two PostToolUse hooks inject context reminders via `additionalContext`:

| Matcher                             | Trigger                | Reminder                                      |
| ----------------------------------- | ---------------------- | --------------------------------------------- |
| `EnterPlanMode`                     | Model enters plan mode | Load `/ora` skill for workflow                |
| `Agent` (filtered `ora:Hephaestus`) | Hephaestus completes   | Run Aletheia verification before squash-merge |

```text
User request
  │
  ▼
Plan Mode ─── hook: load /ora skill ──────────────────
  │  a. Quick landscape scan
  │     Ariadne (codebase) / Clio (external, if needed)
  │  b. If ambiguous → /brainstorm
  │  c. Deep targeted exploration
  │  d. Write plan
  │
  ▼
Execution ─────────────────────────────────────────────
  │  Hephaestus ×N (Opus, parallel worktrees)
  │
  ▼
Verify ─── hook: Aletheia reminder ────────────────────
  │  Aletheia checks acceptance criteria
  │    ├─ VERIFIED   → squash-merge worktree
  │    └─ GAPS_FOUND → resume Hephaestus
  │         └─ still failing → halt, ask user
  │
  ▼
Finalize ──────────────────────────────────────────────
  │  /council-review or /simplify (merge base → HEAD)
  │  Hephaestus fix-wave for scope-heavy issues
  │  git reset --soft <merge-base> + git restore --staged
  │
  ▼
Human reviews changes and commits
```

## Configuration

ora works best with proactive behavioral overrides in `~/.claude/CLAUDE.md`:

```markdown
<investigate_before_responding>
Do not respond to technical tasks without loading a matching skill or
spawning ora:Clio research first. The cost is near zero — the cost of
outdated knowledge is a broken implementation. Skip only when the task
is clearly unrelated to any available skill.
</investigate_before_responding>

<subagent_routing>
Do not use the built-in Explore agent or general-purpose agent — use
ora:Ariadne and ora:Clio instead. General-purpose agent is the escape hatch
when no ora agent fits. Grep/Glob budget at main agent: max 2 calls per task. If
you need a 3rd search, spawn ora:Ariadne instead — batch remaining questions
into the prompt. Do not search alongside Ariadne; delegate fully and wait.
Broad patterns (`**/*.md`, regex with `|` alternation) always go to Ariadne.
</subagent_routing>

<plan_before_implementing>
Do not implement without entering plan mode first. Skip only for truly trivial
tasks (single-file edits, renaming, typo fixes).
</plan_before_implementing>

<delegate_implementation>
Do not Edit/Write 2+ files or >10 lines in main after plan mode exits —
dispatch ora:Hephaestus. Exceptions: typo/comment fixes, unused import removal,
or a 1-line syntax error explicitly pointed out by Aletheia/Hephaestus output.
Any code change requiring reading surrounding context first goes to Hephaestus.
</delegate_implementation>
```

## License

[MIT](./LICENSE) — Cong Tran
