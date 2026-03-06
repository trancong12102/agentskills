---
name: warp-grep
description: "Use this skill for semantic codebase search — when you need to understand how code works, trace data flows across files, or answer conceptual questions about code. Trigger whenever a task requires understanding code intent rather than matching literal text: 'how does X work', 'where is Y configured', 'trace the flow of Z', exploring unfamiliar codebases, debugging cross-file interactions, understanding package relationships, or answering architecture questions. Also trigger when the user asks to trace end-to-end flows, understand how systems are organized, or needs to map out how multiple files/packages interact. Decision rule: if you can write the grep pattern yourself, use Grep; if you need to understand intent or trace multi-file flows, use warp-grep. Do NOT use for simple keyword/symbol searches, file reads, code modifications, or version lookups."
---

# warp-grep

Semantic codebase search powered by MorphLLM's warpgrep — an RL-trained subagent that runs ~15-30 internal grep+read operations to answer natural language questions about code. Unlike simple grep, it understands intent and traces cross-file flows.

Requires `MORPH_API_KEY` environment variable.

## When to Use

- Understanding how a feature works across multiple files (e.g., "how does the auth middleware validate tokens?")
- Tracing data flow end-to-end (e.g., "what happens when a user clicks checkout?")
- Exploring unfamiliar codebases where you don't know what to grep for
- Answering architectural questions (e.g., "how is the database layer organized?")
- Debugging cross-file interactions where the connection isn't obvious

## When NOT to Use

- Simple keyword or symbol searches (e.g., finding all uses of `handleSubmit`) — use Grep
- Finding files by name or pattern — use Glob
- Looking up library documentation — use context7
- Questions unrelated to the current codebase

## Decision Rule

**Can you write the grep pattern yourself?** Use Grep — it's faster.
**Can't write the pattern because the question is conceptual?** Use warp-grep.

| Task | Tool |
|------|------|
| Find all imports of `AuthService` | Grep |
| How does AuthService validate tokens? | warp-grep |
| Find files named `*.config.ts` | Glob |
| How is the config system structured? | warp-grep |
| Find `TODO` comments | Grep |
| What's left unfinished in the payment flow? | warp-grep |

## Workflow

**DO NOT read script source code.** Run scripts directly and use `--help` for usage.

### Run a Semantic Search

```bash
python3 scripts/warp-grep.py search "<natural language query>" [repo_path]
```

- `query` — a natural language question about the code
- `repo_path` — path to the repo root (defaults to current directory)

```bash
# Search the current repo
python3 scripts/warp-grep.py search "how does the authentication flow work"

# Search a specific repo
python3 scripts/warp-grep.py search "how are database migrations handled" /path/to/repo

# Search including node_modules
python3 scripts/warp-grep.py search "how does the router resolve paths" --search-type node_modules

# Increase timeout for large codebases
python3 scripts/warp-grep.py search "trace the payment processing pipeline" --timeout 180
```

Run `python3 scripts/warp-grep.py --help` for full usage.

## Rules

- **Write queries as natural language questions** — `"How does the auth middleware validate JWT tokens?"` works far better than `"auth JWT"`, because warpgrep is an RL-trained agent that plans its own search strategy based on your question.
- **Be specific about what you want to know** — `"What happens when a user submits the settings form?"` beats `"settings form"`. The more context you give, the better it can target its internal searches.
- **Use for understanding, not for finding** — If you already know the symbol or keyword, Grep is faster. warp-grep shines when you don't know what to look for.
- **Default timeout is 120s** — warpgrep runs many internal operations. For large codebases, increase with `--timeout 180` or higher.
