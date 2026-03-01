#!/bin/bash
set -euo pipefail

# Read stdin → extract prompt and session_id
input=$(cat)
prompt=$(echo "$input" | jq -r '.prompt // empty') || exit 0
session_id=$(echo "$input" | jq -r '.session_id // empty') || exit 0
[ -z "$prompt" ] && exit 0

# Check env vars
[ -z "${GEMINI_API_KEY:-}" ] && exit 0
[ -z "${GOOGLE_GEMINI_BASE_URL:-}" ] && exit 0

# Session tracking: load already-recommended skills
session_dir="/tmp/skill-router"
mkdir -p "$session_dir"
session_file="$session_dir/$session_id"
active_skills=""
[ -n "$session_id" ] && [ -f "$session_file" ] && active_skills=$(cat "$session_file")

# Cleanup stale session files (older than 24h)
find "$session_dir" -type f -mtime +1 -delete 2>/dev/null || true

# Build skill catalog from SKILL.md frontmatters
skills_dir="$HOME/.claude/skills"
[ -d "$skills_dir" ] || exit 0

catalog=""
skill_names=""
for skill_md in "$skills_dir"/*/SKILL.md; do
  [ -f "$skill_md" ] || continue
  name=$(yq --front-matter=extract '.name' "$skill_md" 2>/dev/null) || continue
  desc=$(yq --front-matter=extract '.description' "$skill_md" 2>/dev/null) || continue
  [ -z "$name" ] || [ -z "$desc" ] && continue
  catalog="${catalog}- /${name}: ${desc}\n"
  skill_names="${skill_names}${name}\n"
done

[ -z "$catalog" ] && exit 0

# Build system prompt, include active skills exclusion if any
exclude_rule=""
if [ -n "$active_skills" ]; then
  exclude_rule="- EXCLUDE these already-active skills (do NOT recommend them): ${active_skills}"
fi

system_prompt="You are a skill router for Claude Code. Given a user prompt, determine which skills (if any) are relevant.

Available skills:
$(echo -e "$catalog")
Rules:
- Only recommend skills that are clearly relevant to the user's prompt.
- If no skills match, respond with exactly: NONE
- If skills match, respond with ONLY the skill names as a comma-separated list, e.g.: context7, deps-dev
- Do not explain your reasoning. Output only skill names or NONE.
- Be selective — only recommend skills with strong relevance, not tangential matches.
- Maximum 3 skills per recommendation.
${exclude_rule}"

body=$(jq -n \
  --arg system "$system_prompt" \
  --arg prompt "$prompt" \
  '{
    systemInstruction: { parts: [{ text: $system }] },
    contents: [{ role: "user", parts: [{ text: $prompt }] }],
    generationConfig: { maxOutputTokens: 200 }
  }')

# Call Gemini API
base_url="${GOOGLE_GEMINI_BASE_URL%/}"
response=$(curl -s --max-time 10 \
  "${base_url}/v1beta/models/gemini-3-flash-preview:generateContent?key=${GEMINI_API_KEY}" \
  -H "Content-Type: application/json" \
  -d "$body") || exit 0

# Parse response
text=$(echo "$response" | jq -r '.candidates[0].content.parts[0].text // empty' 2>/dev/null) || exit 0
text=$(echo "$text" | tr -d '[:space:]')
[ -z "$text" ] || [ "$text" = "NONE" ] && exit 0

# Validate skill names and format output
valid=""
IFS=',' read -ra items <<< "$text"
for item in "${items[@]}"; do
  name=$(echo "$item" | sed 's|^/||')
  if echo -e "$skill_names" | grep -qx "$name"; then
    valid="${valid:+$valid, }/$name"
  fi
done

[ -z "$valid" ] && exit 0

# Save recommended skills to session file
if [ -n "$session_id" ]; then
  if [ -n "$active_skills" ]; then
    echo "${active_skills}, ${valid}" > "$session_file"
  else
    echo "$valid" > "$session_file"
  fi
fi

echo "[Skill Router] Recommended skills: $valid"
