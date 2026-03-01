# Skill Router

Auto-recommend relevant skills when you submit a prompt in Claude Code. Uses Gemini API to match your prompt against installed skills and suggests the best ones.

## How It Works

1. You type a prompt in Claude Code
2. The hook sends your prompt + installed skill catalog to Gemini
3. Gemini picks the most relevant skills (max 3)
4. You see a recommendation like `[Skill Router] Recommended skills: /context7, /deps-dev`

The plugin is fail-safe — if anything goes wrong (no API key, network error, no skills installed), it silently exits without blocking your prompt.

### Features

- **Per-session dedup**: Skills already recommended in the current session won't be suggested again
- **Validation**: Only recommends skills that are actually installed (hallucinated names are dropped)
- **Fast**: 10-second timeout, uses `gemini-2.5-flash-lite` for low latency

## Prerequisites

- **macOS / Linux** with `bash`
- **[jq](https://jqlang.github.io/jq/)** — JSON processing
- **[yq](https://github.com/mikefarah/yq)** — YAML frontmatter extraction from SKILL.md files
- **curl**
- **Gemini API key** — get one from [Google AI Studio](https://aistudio.google.com/apikey)
- At least one skill installed at `~/.claude/skills/*/SKILL.md`

## Installation

In Claude Code, add the marketplace and install:

```bash
/plugin marketplace add trancong12102/agentskills
/plugin install skill-router@agentskills
```

## Configuration

Set these environment variables (e.g. in your shell profile):

```bash
export GEMINI_API_KEY="your-gemini-api-key"
export GOOGLE_GEMINI_BASE_URL="https://generativelanguage.googleapis.com"
```

## License

[MIT](../../LICENSE) — Cong Tran
