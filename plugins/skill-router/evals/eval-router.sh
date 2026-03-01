#!/bin/bash
set -euo pipefail

# Eval script for skill-router hook
# Usage: ./eval-router.sh [model_name]
# Example: ./eval-router.sh gemini-2.5-flash-lite
#          ./eval-router.sh gemini-3-flash-preview

MODEL="${1:-gemini-2.5-flash-lite}"
RUNS="${2:-1}"
PROMPT_LABEL="${3:-default}"

# Check env vars
if [ -z "${GEMINI_API_KEY:-}" ] || [ -z "${GOOGLE_GEMINI_BASE_URL:-}" ]; then
  echo "ERROR: GEMINI_API_KEY and GOOGLE_GEMINI_BASE_URL must be set"
  exit 1
fi

# Build skill catalog from installed SKILL.md files
skills_dir="$HOME/.claude/skills"
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

if [ -z "$catalog" ]; then
  echo "ERROR: No skills found in $skills_dir"
  exit 1
fi

# System prompt — load from SYSTEM_PROMPT env var or use default
if [ -n "${SYSTEM_PROMPT:-}" ]; then
  system_prompt="${SYSTEM_PROMPT}"
else
  system_prompt="You are a skill router for Claude Code. Given a user prompt, determine which skills (if any) are relevant.

Available skills:
$(echo -e "$catalog")
Rules:
- Only recommend skills that are clearly relevant to the user's prompt.
- If no skills match, respond with exactly: NONE
- If skills match, respond with ONLY the skill names as a comma-separated list, e.g.: context7, deps-dev
- Do not explain your reasoning. Output only skill names or NONE.
- Be selective — only recommend skills with strong relevance, not tangential matches.
- Maximum 3 skills per recommendation."
fi

# Test cases: "prompt|expected_skills" (comma-separated, sorted)
# NONE means no skills should be recommended
test_cases=(
  # === Should trigger specific skills ===
  "what's the latest version of react?|deps-dev"
  "review my PR changes before merging|council-review"
  "look up the Next.js documentation for app router|context7"
  "create a PDF report from this data|pdf"
  "write a Word document with a table of contents|docx"
  "deploy my worker to cloudflare|cloudflare"
  "set up turbo.json for my monorepo|turborepo"
  "implement 2FA with Better Auth|two-factor-authentication-best-practices"
  "scrape data from this website using browser automation|agent-browser"
  "tap the login button on the iOS simulator|agent-device"
  "add TypeScript conditional types to this utility|typescript-advanced-types"
  "audit my website's Core Web Vitals and Lighthouse score|web-perf"
  "set up a Durable Object for a chat room with WebSockets|durable-objects"
  "write tests for my React Native component using testing library|react-native-testing"
  "optimize my React Native app FPS and reduce re-renders|react-native-best-practices"
  "help me build a landing page with beautiful UI|frontend-design"
  "configure wrangler.jsonc for my Workers project|wrangler"
  "how do I set up Better Auth in my Next.js app?|better-auth-best-practices"
  "is there a skill that can help me with database migrations?|NONE"
  "help me write a technical design spec|doc-coauthoring"
  "check my React component for performance anti-patterns|vercel-react-best-practices"
  "how outdated are the packages in my package.json?|deps-dev"
  "set up email and password auth with Better Auth|email-and-password-best-practices"
  "review my UI code for accessibility issues|web-design-guidelines"
  "help me implement organization and RBAC with Better Auth|organization-best-practices"
  # === Should NOT trigger any skills (NONE) ===
  "hello, how are you?|NONE"
  "what is the capital of France?|NONE"
  "fix the typo on line 5|NONE"
  "explain this function to me|NONE"
  "git commit -m 'fix bug'|NONE"
  "refactor this code to use async/await|NONE"
  "add a try-catch block here|NONE"
  "what does this error mean?|NONE"
  "rename this variable to camelCase|NONE"
  "run the test suite|NONE"
  # === Vietnamese prompts - should trigger specific skills ===
  "phiên bản mới nhất của react là gì?|deps-dev"
  "review code trước khi merge PR|council-review"
  "tra cứu docs của Next.js app router|context7"
  "tạo file PDF từ dữ liệu này|pdf"
  "viết file Word có mục lục|docx"
  "deploy worker lên cloudflare|cloudflare"
  "cấu hình turbo.json cho monorepo|turborepo"
  "cài đặt xác thực 2 yếu tố với Better Auth|two-factor-authentication-best-practices"
  "crawl dữ liệu từ website này|agent-browser"
  "bấm nút đăng nhập trên iOS simulator|agent-device"
  "thêm TypeScript conditional types vào utility này|typescript-advanced-types"
  "kiểm tra Core Web Vitals và điểm Lighthouse|web-perf"
  "tạo Durable Object cho phòng chat với WebSocket|durable-objects"
  "viết test cho component React Native|react-native-testing"
  "tối ưu FPS và giảm re-render cho app React Native|react-native-best-practices"
  "giúp tôi làm landing page đẹp|frontend-design"
  "cấu hình wrangler.jsonc cho dự án Workers|wrangler"
  "cách setup Better Auth cho app Next.js?|better-auth-best-practices"
  "có skill nào hỗ trợ database migrations không?|NONE"
  "giúp tôi viết technical design spec|doc-coauthoring"
  "kiểm tra React component có anti-pattern không|vercel-react-best-practices"
  "package.json của tôi outdated chưa?|deps-dev"
  "cài đặt đăng nhập email password với Better Auth|email-and-password-best-practices"
  "review UI code về accessibility|web-design-guidelines"
  "triển khai tổ chức và phân quyền RBAC với Better Auth|organization-best-practices"
  # === Vietnamese prompts - should NOT trigger (NONE) ===
  "xin chào, bạn khỏe không?|NONE"
  "thủ đô của Pháp là gì?|NONE"
  "sửa lỗi chính tả ở dòng 5|NONE"
  "giải thích hàm này cho tôi|NONE"
  "commit code với message fix bug|NONE"
  "refactor đoạn code này sang async/await|NONE"
  "thêm try-catch vào đây|NONE"
  "lỗi này nghĩa là gì?|NONE"
  "đổi tên biến sang camelCase|NONE"
  "chạy test suite|NONE"
)

call_model() {
  local prompt="$1"
  local body
  body=$(jq -n \
    --arg system "$system_prompt" \
    --arg prompt "$prompt" \
    '{
      systemInstruction: { parts: [{ text: $system }] },
      contents: [{ role: "user", parts: [{ text: $prompt }] }],
      generationConfig: { maxOutputTokens: 200 }
    }')

  local base_url="${GOOGLE_GEMINI_BASE_URL%/}"
  local response
  response=$(curl -s --max-time 15 \
    "${base_url}/v1beta/models/${MODEL}:generateContent?key=${GEMINI_API_KEY}" \
    -H "Content-Type: application/json" \
    -d "$body") || { echo "ERROR"; return; }

  local text
  text=$(echo "$response" | jq -r '.candidates[0].content.parts[0].text // "ERROR"' 2>/dev/null) || { echo "ERROR"; return; }
  # Clean whitespace, strip leading slashes
  text=$(echo "$text" | tr -d '[:space:]' | sed 's|/||g')

  # Validate skill names
  if [ "$text" = "NONE" ]; then
    echo "NONE"
    return
  fi

  local valid=""
  IFS=',' read -ra items <<< "$text"
  for item in "${items[@]}"; do
    local name
    name=$(echo "$item" | xargs)
    if echo -e "$skill_names" | grep -qx "$name"; then
      valid="${valid:+$valid,}$name"
    fi
  done

  if [ -z "$valid" ]; then
    echo "NONE"
  else
    # Sort for consistent comparison
    echo "$valid" | tr ',' '\n' | sort | tr '\n' ',' | sed 's/,$//'
  fi
}

normalize() {
  echo "$1" | tr ',' '\n' | sort | tr '\n' ',' | sed 's/,$//'
}

# Run eval
echo "========================================"
echo "Skill Router Eval"
echo "Model: $MODEL"
echo "Prompt: $PROMPT_LABEL"
echo "Runs: $RUNS"
echo "Test cases: ${#test_cases[@]}"
echo "========================================"
echo

total=0
exact_match=0
tp_total=0
fp_total=0
fn_total=0
errors=0
latency_sum=0

for run in $(seq 1 "$RUNS"); do
  [ "$RUNS" -gt 1 ] && echo "--- Run $run/$RUNS ---"

  for tc in "${test_cases[@]}"; do
    prompt="${tc%%|*}"
    expected_raw="${tc##*|}"
    expected=$(normalize "$expected_raw")

    start_ms=$(($(date +%s%N 2>/dev/null || gdate +%s%N) / 1000000))
    got=$(call_model "$prompt")
    end_ms=$(($(date +%s%N 2>/dev/null || gdate +%s%N) / 1000000))
    latency=$((end_ms - start_ms))
    latency_sum=$((latency_sum + latency))

    got_sorted=$(normalize "$got")
    total=$((total + 1))

    if [ "$got" = "ERROR" ]; then
      errors=$((errors + 1))
      status="ERR"
    elif [ "$got_sorted" = "$expected" ]; then
      exact_match=$((exact_match + 1))
      status="OK "
    else
      status="FAIL"
    fi

    # Calculate TP/FP/FN
    if [ "$got" != "ERROR" ]; then
      if [ "$expected" = "NONE" ] && [ "$got_sorted" = "NONE" ]; then
        tp_total=$((tp_total + 1))
      elif [ "$expected" = "NONE" ] && [ "$got_sorted" != "NONE" ]; then
        # False positives: all recommended skills
        fp_count=$(echo "$got_sorted" | tr ',' '\n' | wc -l | xargs)
        fp_total=$((fp_total + fp_count))
      elif [ "$expected" != "NONE" ] && [ "$got_sorted" = "NONE" ]; then
        # False negatives: all expected skills
        fn_count=$(echo "$expected" | tr ',' '\n' | wc -l | xargs)
        fn_total=$((fn_total + fn_count))
      else
        # Both have skills — compare sets
        IFS=',' read -ra exp_arr <<< "$expected"
        IFS=',' read -ra got_arr <<< "$got_sorted"
        for e in "${exp_arr[@]}"; do
          if echo "$got_sorted" | grep -qw "$e"; then
            tp_total=$((tp_total + 1))
          else
            fn_total=$((fn_total + 1))
          fi
        done
        for g in "${got_arr[@]}"; do
          if ! echo "$expected" | grep -qw "$g"; then
            fp_total=$((fp_total + 1))
          fi
        done
      fi
    fi

    # Truncate prompt for display
    short_prompt="${prompt:0:50}"
    [ ${#prompt} -gt 50 ] && short_prompt="${short_prompt}..."

    if [ "$status" = "OK " ]; then
      printf "[%s] %4dms %-55s → %s\n" "$status" "$latency" "$short_prompt" "$got_sorted"
    else
      printf "[%s] %4dms %-55s → got: %-30s expected: %s\n" "$status" "$latency" "$short_prompt" "$got_sorted" "$expected"
    fi
  done
done

# Summary
echo
echo "========================================"
echo "Results: $MODEL"
echo "========================================"
accuracy=$(echo "scale=1; $exact_match * 100 / $total" | bc)
avg_latency=$((latency_sum / total))

precision="N/A"
recall="N/A"
f1="N/A"
if [ $((tp_total + fp_total)) -gt 0 ]; then
  precision=$(echo "scale=1; $tp_total * 100 / ($tp_total + $fp_total)" | bc)
fi
if [ $((tp_total + fn_total)) -gt 0 ]; then
  recall=$(echo "scale=1; $tp_total * 100 / ($tp_total + $fn_total)" | bc)
fi
if [ "$precision" != "N/A" ] && [ "$recall" != "N/A" ]; then
  p=$(echo "scale=4; $tp_total / ($tp_total + $fp_total)" | bc)
  r=$(echo "scale=4; $tp_total / ($tp_total + $fn_total)" | bc)
  sum=$(echo "scale=4; $p + $r" | bc)
  if [ "$(echo "$sum > 0" | bc)" -eq 1 ]; then
    f1=$(echo "scale=1; 2 * $p * $r / $sum * 100" | bc)
  fi
fi

echo "Exact match:  $exact_match/$total ($accuracy%)"
echo "Precision:    $precision%  (TP=$tp_total, FP=$fp_total)"
echo "Recall:       $recall%  (TP=$tp_total, FN=$fn_total)"
echo "F1 Score:     $f1%"
echo "Errors:       $errors"
echo "Avg latency:  ${avg_latency}ms"
echo "========================================"
