---
name: codebase-search
description: "Semantic codebase search — use as the first step when exploring or scanning a codebase broadly. Understands code intent and traces cross-file flows. Use when the user needs to understand how code works, trace data flows, explore unfamiliar codebases, or answer conceptual questions ('how does X work', 'where is Y configured', 'trace the flow of Z'). Do NOT use for targeted keyword/symbol searches (use Grep) or file lookups (use Glob)."
---

# codebase-search

Semantic codebase search powered by MorphLLM — an RL-trained subagent that runs ~15-30 internal grep+read operations to answer natural language questions about code. Unlike simple grep, it understands intent and traces cross-file flows.

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
**Can't write the pattern because the question is conceptual?** Use codebase-search.

| Task                                        | Tool            |
| ------------------------------------------- | --------------- |
| Find all imports of `AuthService`           | Grep            |
| How does AuthService validate tokens?       | codebase-search |
| Find files named `*.config.ts`              | Glob            |
| How is the config system structured?        | codebase-search |
| Find `TODO` comments                        | Grep            |
| What's left unfinished in the payment flow? | codebase-search |

## Workflow

**DO NOT read script source code.** Run scripts directly and use `--help` for usage.

### Run a Semantic Search

```bash
python3 scripts/codebase-search.py search "<natural language query>" [repo_path]
```

- `query` — a natural language question about the code
- `repo_path` — path to the repo root (defaults to current directory)

```bash
# Search the current repo
python3 scripts/codebase-search.py search "how does the authentication flow work"

# Search a specific repo
python3 scripts/codebase-search.py search "how are database migrations handled" /path/to/repo

# Search including node_modules
python3 scripts/codebase-search.py search "how does the router resolve paths" --search-type node_modules

# Increase timeout for large codebases
python3 scripts/codebase-search.py search "trace the payment processing pipeline" --timeout 180
```

Run `python3 scripts/codebase-search.py --help` for full usage.

### Example Output

```text
Morph Fast Context subagent performed search on repository:

Relevant context found:
- plugins/ora/agents/ariadne.md:*
- plugins/ora/agents/clio.md:*
- plugins/ora/agents/metis.md:*

Here is the content of files:

<file path="plugins/ora/agents/ariadne.md">
1| ---
2| name: Ariadne
3| description: |
4|   Use this agent to explore and understand codebases...
...
35| tools: ["Read", "Glob", "Grep", "LSP", "Bash"]
36| skills:
37|   - codebase-search
38| ---
</file>

<file path="plugins/ora/agents/clio.md">
1| ---
2| name: Clio
...
</file>
```

The output lists relevant files found and their full content. Use these results directly — do not re-search the same files with Grep/Read.

## Rules

- **Write queries as natural language questions** — `"How does the auth middleware validate JWT tokens?"` works far better than `"auth JWT"`, because codebase search is an RL-trained agent that plans its own search strategy based on your question.
- **Be specific about what you want to know** — `"What happens when a user submits the settings form?"` beats `"settings form"`. The more context you give, the better it can target its internal searches.
- **Use for understanding, not for finding** — If you already know the symbol or keyword, Grep is faster. codebase-search shines when you don't know what to look for.
- **Default timeout is 120s** — codebase search runs many internal operations. For large codebases, increase with `--timeout 180` or higher.
