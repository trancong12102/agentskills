# codebase-search

Semantic local search powered by MorphLLM — an RL-trained subagent that runs ~15-30 internal grep+read operations to answer natural language questions about any project files (code, config, docs, markdown, YAML, JSON, and other text files). Unlike simple grep, it understands intent and traces cross-file flows.

Requires `MORPH_API_KEY` environment variable.

## When to Use

- Understanding how a feature works across multiple files (e.g., "how does the auth middleware validate tokens?")
- Tracing data flow end-to-end (e.g., "what happens when a user clicks checkout?")
- Exploring unfamiliar projects where you don't know what to grep for
- Answering architectural questions (e.g., "how is the database layer organized?")
- Debugging cross-file interactions where the connection isn't obvious
- Finding information in non-code files — config, docs, markdown, YAML, JSON (e.g., "how are the plugin hooks configured?")

## When NOT to Use

- Simple keyword or symbol searches (e.g., finding all uses of `handleSubmit`) — use `fff` MCP grep, or shell `rg`/`grep`/`ugrep` as fallback
- Finding files by name or pattern — use `fff` MCP find_files, or shell `fd`/`find`/`bfs` as fallback
- Looking up library documentation — use context7
- Questions unrelated to the current codebase

## Decision Rule

**Can you write the grep pattern yourself?** Use `fff` MCP grep (or whichever shell grep tool — `rg`/`grep`/`ugrep` — is installed, as fallback). Faster than codebase-search.
**Can't write the pattern because the question is conceptual?** Use codebase-search.

Shell tools (`rg`/`grep`/`ugrep`, `fd`/`find`/`bfs`) sit below both codebase-search and `fff` — only reach for them when fff is unavailable or the target is outside the git index.

| Task                                        | Tool                                           |
| ------------------------------------------- | ---------------------------------------------- |
| Find all imports of `AuthService`           | `fff` grep (fallback: `rg`/`grep`/`ugrep`)     |
| How does AuthService validate tokens?       | codebase-search                                |
| Find files named `*.config.ts`              | `fff` find_files (fallback: `fd`/`find`/`bfs`) |
| How is the config system structured?        | codebase-search                                |
| Find `TODO` comments                        | `fff` grep (fallback: `rg`/`grep`/`ugrep`)     |
| What's left unfinished in the payment flow? | codebase-search                                |

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
35| tools: ["Read", "LSP", "Bash"]
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

The output lists relevant files found and their full content. Use these results directly — do not re-search the same files with shell grep or Read.

## Rules

- **Write queries as natural language questions** — `"How does the auth middleware validate JWT tokens?"` works far better than `"auth JWT"`, because codebase search is an RL-trained agent that plans its own search strategy based on your question.
- **Be specific about what you want to know** — `"What happens when a user submits the settings form?"` beats `"settings form"`. The more context you give, the better it can target its internal searches.
- **Use for understanding, not for finding** — If you already know the symbol or keyword, `fff` grep (or shell `rg`/`grep`/`ugrep` as fallback) is faster. codebase-search shines when you don't know what to look for.
- **Default timeout is 120s** — codebase search runs many internal operations. For large codebases, increase with `--timeout 180` or higher.
