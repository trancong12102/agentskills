# Context7

Retrieve current documentation for software libraries via the official `ctx7` CLI. This is especially useful when you're unsure about an API's current interface — library docs change frequently and your training data may be outdated.

Auth is handled by the CLI itself. Run `bunx ctx7@latest login` once; verify with `bunx ctx7@latest whoami`. No API key env var.

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

### Step 1: Resolve the library ID

```bash
bunx ctx7@latest library <name> [query]
```

Lists candidate libraries with Context7 IDs (e.g. `/websites/react_dev`), trust scores, snippet counts, and benchmark scores. The optional `[query]` re-ranks results by relevance. Pick the highest-trust ID that matches the library you want.

Add `--json` for machine-readable output:

```bash
bunx ctx7@latest library react "useState hook" --json
```

### Step 2: Query documentation

```bash
bunx ctx7@latest docs <libraryId> "<query>"
```

Returns markdown snippets ordered by relevance to your query.

If the first answer is shallow or wrong, retry with `--research`:

```bash
bunx ctx7@latest docs /websites/react_dev "useState hook" --research
```

`--research` spins up sandboxed agents that pull the source repo, inspect it, and run a live web search before synthesizing. It's slower and more expensive — use as a retry, not by default.

Run `bunx ctx7@latest --help`, `bunx ctx7@latest library --help`, or `bunx ctx7@latest docs --help` for full usage.

## Examples

```bash
# Find React library ID, then fetch useState docs
bunx ctx7@latest library react "useState hook"
bunx ctx7@latest docs /websites/react_dev "useState hook with objects"

# Quick API check
bunx ctx7@latest docs /vercel/next.js "middleware redirect"

# Retry with deep research when the default answer was insufficient
bunx ctx7@latest docs /langchain-ai/langchainjs "retrieval chain setup" --research
```

## Rules

- **Write specific queries** — `"useState hook with objects"` retrieves much better results than `"hooks"`, because the API ranks snippets by relevance to your query. Pass a query to `library` too — it improves ID ranking.
- **Always resolve library ID first** — IDs are not guessable (e.g., `/websites/react_dev` vs `/facebook/react`), and the wrong ID returns the wrong corpus.
- **Use `--research` as a retry, not default** — it pulls source repos and runs a live web search; reserve it for follow-ups when the default answer was shallow.
