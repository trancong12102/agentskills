---
name: council-oracle
description: "Use this skill when the user needs deep analysis, reasoning, or expert guidance on complex problems. Trigger on: 'oracle', 'think about this', 'analyze this deeply', 'second opinion', 'deep dive', architecture analysis, elusive bug debugging, impact assessment, security reasoning, refactoring strategy, trade-off evaluation, or complex why/how questions. Runs Codex and Claude in parallel, then synthesizes. Do NOT use for simple factual questions, code generation, code review (use council-review), or tasks needing file modifications."
---

# Council Oracle

Run Codex and a Claude subagent in parallel for deep analysis, then cross-validate and synthesize into one unified report — like a panel of two senior architects independently analyzing a problem, with Claude as lead synthesizer delivering the final assessment.

## Prerequisites

- **Codex CLI**: Install with `npm i -g @openai/codex`, authenticate with `codex login`

If only one CLI is installed, fall back to the available oracle with a warning — the analysis still has value with fewer perspectives, so don't fail entirely.

## When to Use

- Architecture analysis and design decisions
- Debugging elusive or complex bugs
- Impact assessment of proposed changes
- Security reasoning and threat modeling
- Refactoring strategy evaluation
- Trade-off analysis between approaches
- Complex "why" or "how" questions about a codebase

## When NOT to Use

- Simple factual questions (just answer directly)
- Code generation tasks (just write the code)
- Code review (use `council-review` instead)
- Tasks that require file modifications (oracle is read-only analysis)

## Workflow

**DO NOT read script source code.** Run scripts directly and use `--help` for usage.

### Step 1: Gather Context

Before launching the oracles, Claude must gather relevant context because Codex CLI only sees what's explicitly passed to it.

1. Understand the user's question and identify what parts of the codebase are relevant.
2. Use Read, Grep, and Glob tools to read relevant files and code snippets.
3. **Research official sources before deciding** — Don't rely solely on training knowledge. Proactively:
   - Use the `context7` skill to fetch up-to-date official documentation for any libraries, frameworks, or tools involved in the question.
   - Use WebSearch to find official blog posts, best practices guides, RFCs, or authoritative references that inform the analysis.
   - Include the findings as additional context files so both oracles benefit from accurate, current information.
4. Write the gathered context to temporary files that can be passed to the scripts via `--context-file`.
5. Formulate a clear, specific question that captures the user's intent.

### Step 2: Run Both Oracles in Parallel

Both oracles analyze the same question independently — neither depends on the other's output. Launch them both at once in a single message to eliminate sequential wait time.

Scripts are in `scripts/` relative to this skill's directory and enforce the correct model and read-only mode internally. Run `<script> --help` for full usage.

#### Codex — `scripts/codex-oracle.py` (background Bash task)

Launch as a background Bash task (`run_in_background: true`). **Codex CLI thinks deeply and may take up to 30 minutes** — do not treat a long wait as a failure. You will be notified automatically when it completes.

```bash
python3 scripts/codex-oracle.py --question "..." --context-file path1 --context-file path2 [--focus text] [--dry-run]
```

#### Claude — Agent tool (foreground)

While Codex runs in the background, launch a Claude subagent using the Agent tool with `subagent_type="general-purpose"`. The subagent should:

- Receive the same question and context
- Be instructed to use only read-only tools (Read, Grep, Glob, WebSearch, WebFetch)
- Analyze the question deeply from a senior software architect perspective
- Return structured analysis with: summary, key findings (with confidence and category), recommendations (with priority), and risks

### Step 3: Cross-Validate

Once both oracle responses have returned, cross-validate:

1. **Validate external findings** — For each finding from Codex:
   - **Confirm** — Claude independently agrees the finding is valid and accurate.
   - **Dispute** — Claude believes the finding is incorrect or irrelevant. Note the reasoning.
   - **Enhance** — The finding is valid but the explanation or recommendation can be improved. Provide the improved version.
2. **Add Claude's unique insights** — Include any findings from the Claude subagent that Codex didn't identify.

### Step 4: Synthesize into Unified Report

After cross-validation is complete, **merge, deduplicate, and rewrite** all findings into one coherent report as if written by a single analyst. Do not copy-paste or concatenate raw outputs.

Load `references/output-format.md` for the report template. Load `references/synthesis-rules.md` for how to reconcile findings across oracles.

## Rules

- **Run both oracles in parallel** — Codex and the Claude subagent are independent analyses of the same question. Running them concurrently saves significant time.
- **Use the same question and context for both oracles** — comparing analyses of different questions would make synthesis meaningless.
- **Wait for both oracles before synthesizing** — Claude's cross-validation is what turns two outputs into one trustworthy report. Both must complete before synthesis begins.
- **Write one unified analysis** — the report should read as a single analyst's assessment. Never structure findings by oracle source (no "Codex found..." sections).
- **Organize findings by theme** — group related insights together, not by source or severity alone. Structure adapts to question type (architecture -> components/trade-offs, bug -> root cause hypotheses, security -> threat model, etc.).
- **Research before reasoning** — Always check official documentation (via `context7`) and search for best practices, official blogs, and authoritative references (via WebSearch) before forming conclusions. Decisions grounded in current, official sources are far more trustworthy than those based on training knowledge alone.
- **Always use the wrapper script** for Codex — never call `codex` CLI directly, because the script sets the correct model and read-only mode.
- If Codex CLI is missing, run the Claude subagent alone and synthesize normally.
