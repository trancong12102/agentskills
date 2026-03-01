#!/bin/bash
set -euo pipefail

# Eval multiple prompt variants against the same model
# Usage: ./eval-prompts.sh [model_name]

MODEL="${1:-gemini-2.5-flash-lite}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Build catalog (shared across prompts)
skills_dir="$HOME/.claude/skills"
catalog=""
for skill_md in "$skills_dir"/*/SKILL.md; do
  [ -f "$skill_md" ] || continue
  name=$(yq --front-matter=extract '.name' "$skill_md" 2>/dev/null) || continue
  desc=$(yq --front-matter=extract '.description' "$skill_md" 2>/dev/null) || continue
  [ -z "$name" ] || [ -z "$desc" ] && continue
  catalog="${catalog}- /${name}: ${desc}\n"
done

# ──────────────────────────────────────────────
# Prompt A: Current (baseline)
# ──────────────────────────────────────────────
PROMPT_A="You are a skill router for Claude Code. Given a user prompt, determine which skills (if any) are relevant.

Available skills:
$(echo -e "$catalog")
Rules:
- Only recommend skills that are clearly relevant to the user's prompt.
- If no skills match, respond with exactly: NONE
- If skills match, respond with ONLY the skill names as a comma-separated list, e.g.: context7, deps-dev
- Do not explain your reasoning. Output only skill names or NONE.
- Be selective — only recommend skills with strong relevance, not tangential matches.
- Maximum 3 skills per recommendation."

# ──────────────────────────────────────────────
# Prompt B: Improved — prefer specific, add examples
# ──────────────────────────────────────────────
PROMPT_B="You are a skill router. Given a user prompt, pick the most relevant skill(s).

Available skills:
$(echo -e "$catalog")
Rules:
- Output ONLY skill names (comma-separated) or NONE. No explanation.
- Recommend the SINGLE most specific skill. Only add a second skill if it covers a clearly DIFFERENT aspect.
- If a specialized skill matches, do NOT also recommend its parent/general skill.
- Maximum 2 skills.

Examples:
- \"set up 2FA with Better Auth\" → two-factor-authentication-best-practices (NOT also better-auth-best-practices)
- \"deploy my worker to cloudflare\" → cloudflare (NOT also wrangler, workers-best-practices)
- \"build a landing page\" → frontend-design (NOT also ui-ux-pro-max, theme-factory)
- \"fix this typo\" → NONE"

# ──────────────────────────────────────────────
# Prompt C: Minimal — ultra-concise, rely on examples
# ──────────────────────────────────────────────
PROMPT_C="Route user prompts to skills. Output skill names (comma-separated) or NONE. No explanation. Pick only the single best-matching skill unless two cover clearly different needs. Max 2.

Skills:
$(echo -e "$catalog")
Key rules:
- Prefer the most SPECIFIC skill over general ones.
- General coding tasks (fix bug, refactor, explain) → NONE

Examples:
- \"2FA with Better Auth\" → two-factor-authentication-best-practices
- \"deploy to cloudflare\" → cloudflare
- \"landing page\" → frontend-design
- \"latest version of react\" → deps-dev
- \"review my PR\" → council-review
- \"hello\" → NONE"

# Run evals
echo "============================================================"
echo "Comparing prompt variants on model: $MODEL"
echo "============================================================"
echo

for label in A B C; do
  var="PROMPT_${label}"
  echo ">>> Running Prompt $label ..."
  SYSTEM_PROMPT="${!var}" "$SCRIPT_DIR/eval-router.sh" "$MODEL" 1 "prompt-$label" 2>&1
  echo
done
