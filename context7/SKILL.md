---
name: context7
description: "Use this skill to fetch up-to-date documentation for any open-source library or framework. Trigger whenever the user asks to look up library docs, check an API, pull up documentation, find code examples, or verify how a library feature works — especially if they mention a specific library name, version migration, outdated docs, or say things like 'what's the current way to...' or 'the API might have changed'. Also trigger when installing or configuring a library. Do NOT use for general programming concepts, internal project code, or questions unrelated to a specific library."
---

# Context7

Retrieve current documentation for software libraries by querying the Context7 API. This is especially useful when you're unsure about an API's current interface — library docs change frequently and your training data may be outdated.

Requires `CONTEXT7_API_KEY` environment variable.

## When to Use

- Looking up how a library function works (e.g., "how does `useEffect` cleanup work?")
- Checking if an API has changed in a newer version
- Finding code examples for a specific library feature
- Verifying correct import paths or function signatures
- Installing or configuring a library and needing setup docs

## When NOT to Use

- General programming questions (e.g., "how do closures work in JS?")
- Questions about your own project's internal code
- Topics unrelated to a specific open-source library or framework

## Workflow

**DO NOT read script source code.** Run scripts directly and use `--help` for usage.

### Step 1: Search for the Library

```bash
python3 scripts/context7.py search <library> <topic>
```

Returns TSV with top 5 matches: `id`, `title`, `snippets`. Use the `id` from the best-matching row for the fetch step.

### Step 2: Fetch Documentation

```bash
python3 scripts/context7.py fetch <library_id> <topic> [--max-tokens N]
```

Fetches documentation snippets relevant to the topic, truncated to a token budget (default: 5000).

**Choosing `--max-tokens`:**

| Scenario | Tokens | Why |
|----------|--------|-----|
| Quick lookup (one function signature) | 2000 | Keeps output focused, faster response |
| Typical usage (API patterns, examples) | 5000 (default) | Good balance of depth and brevity |
| Broad exploration (migration guide, full API surface) | 8000–10000 | Needed when topic spans multiple sections |

Run `python3 scripts/context7.py --help` for full usage.

## Examples

```bash
# Find React library ID, then fetch useState docs
python3 scripts/context7.py search react "useState hook"
python3 scripts/context7.py fetch /websites/react_dev "useState hook with objects"

# Smaller budget for a quick lookup
python3 scripts/context7.py fetch /vercel/next.js "middleware redirect" --max-tokens 2000

# Broader exploration
python3 scripts/context7.py fetch /langchain-ai/langchainjs "retrieval chain setup" --max-tokens 8000
```

## Rules

- **Write specific queries** — `"useState hook with objects"` retrieves much better results than `"hooks"`, because the API ranks snippets by relevance to your query.
- **Always search before fetching** — Library IDs aren't guessable (e.g., `/websites/react_dev`), so you need the search step to find the right one.
- **Match `--max-tokens` to the task** — Use the table above. Overshooting wastes context window; undershooting may miss the answer.
