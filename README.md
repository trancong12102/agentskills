# Agent Skills

Reusable skills and agents for AI coding agents, primarily Claude Code.

## Why ora

Claude Code is capable out of the box, but on complex tasks it tends to jump straight into code, skip pre-analysis, and lose track of multi-step plans. ora adds structure where it matters — before planning starts and after execution ends — without getting in the way for simple tasks.

The core workflow is a loop-based pipeline where each stage iterates until satisfied:

1. **Analyze** — classify intent, explore the codebase, surface risks, gather missing context via external research or user clarification.
2. **Plan** — write a plan informed by the analysis directives.
3. **Validate** — review the plan for executability. If rejected, fix and re-review — same session, no context lost.
4. **Dispatch** — group tasks into parallel waves with dependency ordering and learning carry-forward.
5. **Execute** — run tasks in isolated worktrees, each independently verified before merging.

Every stage uses **session resume** (SendMessage) instead of respawning, preserving full context and saving ~70% tokens per round-trip.

### Comparison

|                      | ora                                       | [superpowers]                | [get-shit-done]                 | [oh-my-openagent]                          |
| -------------------- | ----------------------------------------- | ---------------------------- | ------------------------------- | ------------------------------------------ |
| Architecture         | 7 agents, multi-model                     | 14 skills, single model      | 24 agents, single model         | 11 agents, multi-model                     |
| Runtime deps         | Zero                                      | Zero                         | Zero (Node >= 22)               | Heavy (ast-grep, MCP SDK, native binaries) |
| Token strategy       | Session resume (~70% savings on retries)  | On-demand skill loading      | File-based context (.planning/) | Always-on MCPs + injected                  |
| Workflow enforcement | 2 hooks (1 blocking, 1 advisory)          | 1 hook (soft)                | 9 hooks (mostly advisory)       | Behavioral (intent gate, todo enforcer)    |
| Session management   | Resume via SendMessage                    | Fresh subagent per task      | File-based persistence          | Auto-recovery, no explicit resume          |
| Composability        | Skills via frontmatter, agents composable | Skills loaded via Skill tool | Edit agent markdown files       | JSONC config, prompt_append                |
| Host                 | Claude Code                               | Claude Code + 7 others       | 13 runtimes                     | OpenCode (Claude Code compat layer)        |
| License              | MIT                                       | MIT                          | MIT                             | SUL-1.0                                    |

[superpowers]: https://github.com/obra/superpowers
[get-shit-done]: https://github.com/gsd-build/get-shit-done
[oh-my-openagent]: https://github.com/code-yeongyu/oh-my-openagent

**Where ora fits**: no compiled plugins, no native binaries, no mandatory MCP servers — just markdown agents and shell scripts. Skills are composable via frontmatter, agents have non-overlapping tool access, and the pipeline is opt-in per step.

## Getting Started

### Install

```shell
# Plugin (agents + hooks)
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

| Agent            | Model  | Role                                                                              |
| ---------------- | ------ | --------------------------------------------------------------------------------- |
| `ora:Metis`      | Opus   | Pre-analysis — classifies intent, surfaces risks, gathers context iteratively.    |
| `ora:Momus`      | Sonnet | Plan review — checks executability, rejects only for true blockers.               |
| `ora:Atlas`      | Opus   | Wave dispatch — groups tasks into parallel waves with learning carry-forward.     |
| `ora:Hephaestus` | Opus   | Deep worker — implements in isolated worktrees, commits before returning.         |
| `ora:Aletheia`   | Sonnet | Verification — checks acceptance criteria against actual code, not summaries.     |
| `ora:Ariadne`    | Sonnet | Codebase exploration — traces flows, finds implementations, maps architecture.    |
| `ora:Clio`       | Sonnet | External research — fetches docs, searches GitHub repos, checks package versions. |

### Workflow

All stages resume via SendMessage instead of respawning. Ariadne and Clio are spawned on-demand throughout.

```text
User request
  │
  ▼
Metis loop (max 3 rounds) ──────────────────────────────
  │  Analyze → status:
  │    READY        → proceed
  │    NEED_RESEARCH → Clio → resume Metis
  │    NEED_USER    → AskUserQuestion → resume Metis
  │
  ▼
EnterPlanMode ───────────────────────────────────────────
  │  Hook: check-metis.sh (blocks if Metis not called)
  │
  ▼
Plan mode ───────────────────────────────────────────────
  │
  ▼
Momus loop (max 3 rounds) ──────────────────────────────
  │  Review → verdict:
  │    OKAY   → proceed
  │    REJECT → fix plan → resume Momus
  │
  ▼
Atlas ───────────────────────────────────────────────────
  │  Wave dispatch (resume if user modifies)
  │
  ▼
ExitPlanMode ────────────────────────────────────────────
  │  Hook: check-plan-review.sh (reminds if review skipped)
  │
  ▼
Execution (per wave) ───────────────────────────────────
  │  Hephaestus — code tasks in isolated worktrees
  │  Ariadne    — codebase exploration
  │  Clio       — external research
  │
  ▼
Verify-Correct loop (per task, max 2 retries) ──────────
     Aletheia checks acceptance criteria
       ├─ VERIFIED   → merge worktree
       └─ GAPS_FOUND → resume Hephaestus
            └─ still failing → halt, ask user
```

### Hooks

| Hook                       | Script                 | Behavior                                        |
| -------------------------- | ---------------------- | ----------------------------------------------- |
| `PreToolUse:EnterPlanMode` | `check-metis.sh`       | **Blocks** if Metis was not spawned.            |
| `PreToolUse:ExitPlanMode`  | `check-plan-review.sh` | **Reminds** if Momus or Atlas were not spawned. |

## Configuration

ora works best with proactive behavioral overrides in `~/.claude/CLAUDE.md`:

```markdown
<investigate_before_responding>
Do not respond to technical tasks using training data alone — verify with a
matching skill or ora:Clio research first.
</investigate_before_responding>

<use_skills_proactively>
Do not write code or give technical advice without first loading a matching
skill. Skip only for git operations, file renaming, simple config edits.
</use_skills_proactively>

<subagent_routing>
Do not use the built-in Explore agent or general-purpose agent if an ora agent
can handle the task.
</subagent_routing>

<plan_before_implementing>
Do not implement without entering plan mode first. Skip only for truly trivial
tasks (single-file edits, renaming, typo fixes).
</plan_before_implementing>

<workflow>
All agents use resume via SendMessage — do not respawn when the same session
can continue.

1. Metis loop (max 3 rounds). Spawn ora:Metis. On return:
   - READY → enter plan mode.
   - NEED_RESEARCH → Clio → resume Metis.
   - NEED_USER → AskUserQuestion → resume Metis.
2. Enter plan mode with Metis directives.
3. Momus loop (max 3 rounds). Spawn ora:Momus with plan. On return:
   - OKAY → proceed.
   - REJECT → fix → resume Momus.
4. Atlas. Spawn ora:Atlas. Resume if user modifies dispatch.
5. Exit plan mode. Execute waves per Atlas dispatch.
6. Verify-correct loop (max 2 retries). Aletheia per Hephaestus task.
   GAPS_FOUND → resume Hephaestus. Still failing → ask user.

</workflow>
```

## License

[MIT](./LICENSE) — Cong Tran
