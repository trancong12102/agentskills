---
name: context7
description: Retrieve up-to-date documentation for software libraries and frameworks via the Context7 API. Use when looking up library documentation, finding code examples, or verifying correct usage of library APIs.
---

# Context7

Retrieve current documentation for software libraries by querying the Context7 API. Requires `CONTEXT7_API_KEY` environment variable.

## Workflow

### Step 1: Search for the Library

```bash
scripts/context7.sh search <library> <topic>
```

Returns TSV with top 5 matches: `id`, `title`, `snippets`. Use the `id` from the first row for the fetch step.

### Step 2: Fetch Documentation

```bash
scripts/context7.sh fetch <library_id> <topic> [--max-tokens N]
```

Fetches documentation snippets relevant to the topic, truncated to a token budget (default: 5000). Only the most relevant snippets are returned. Use `--max-tokens` to control output size — lower values for focused lookups, higher for broad exploration.

Run `scripts/context7.sh --help` for full usage.

## Examples

```bash
# Find React library ID, then fetch useState docs
scripts/context7.sh search react "useState hook"
scripts/context7.sh fetch /websites/react_dev "useState hook with objects"

# Smaller budget for a quick lookup
scripts/context7.sh fetch /vercel/next.js "middleware redirect" --max-tokens 2000
```

## Rules

- Write specific queries — `"useState hook with objects"` is better than `"hooks"`
- Always search first to get the correct `library_id` before fetching
- Use `--max-tokens 2000-3000` for focused lookups, default 5000 for broader topics
