---
name: github-codebase-search
description: "Semantic search for public GitHub repos without cloning. Use when the user wants to understand how an external library or framework works internally, investigate upstream bugs, trace code paths in a repo they haven't cloned, or search GitHub source code by intent. Do NOT use for local codebase questions (use codebase-search), documentation lookup (use context7), or private repos."
---

# github-codebase-search

Semantic search for public GitHub repositories powered by MorphLLM — an RL-trained subagent that runs parallel grep+read calls against the GitHub API to answer natural language questions about code. No cloning required.

Requires `MORPH_API_KEY` environment variable.

## When to Use

- Understanding how a library or framework works internally (e.g., "how does Next.js resolve middleware?")
- Investigating upstream bugs or behavior in open-source dependencies
- Learning from reference implementations in well-known repos
- Tracing code paths in a repo you haven't cloned locally
- Exploring how an OSS project is structured or architected

## When NOT to Use

- Questions about the local codebase — use codebase-search
- Private repositories (GitHub API access required)
- Simple documentation lookup — use context7
- Simple GitHub UI browsing (viewing READMEs, issues, PRs)

## Decision Rule

**Is the code on GitHub and not cloned locally?** Use github-codebase-search.
**Is it in the local repo?** Use codebase-search.

| Task                                          | Tool                   |
| --------------------------------------------- | ---------------------- |
| How does Next.js resolve API routes?          | github-codebase-search |
| How does our auth middleware work?            | codebase-search        |
| How does Prisma handle migrations internally? | github-codebase-search |
| Where is the database config in our repo?     | codebase-search        |
| What does React's reconciler do with fibers?  | github-codebase-search |
| What does the React docs say about useEffect? | context7               |

## Workflow

**DO NOT read script source code.** Run scripts directly and use `--help` for usage.

### Run a Semantic Search

```bash
python3 scripts/github-codebase-search.py search "<natural language query>" --repo <owner/repo>
```

- `query` — a natural language question about the code
- `--repo` — GitHub repository in `owner/repo` format (e.g., `vercel/next.js`)
- `--url` — alternative: full GitHub URL (e.g., `https://github.com/vercel/next.js`)
- Must provide either `--repo` or `--url`

```bash
# Search by owner/repo
python3 scripts/github-codebase-search.py search "how does the router resolve middleware" --repo vercel/next.js

# Search by GitHub URL
python3 scripts/github-codebase-search.py search "how does the router resolve middleware" --url https://github.com/vercel/next.js

# Search a specific branch
python3 scripts/github-codebase-search.py search "how are migrations handled" --repo prisma/prisma --branch main

# Increase timeout for large repos
python3 scripts/github-codebase-search.py search "trace the build pipeline" --repo facebook/react --timeout 180
```

Run `python3 scripts/github-codebase-search.py --help` for full usage.

### Example Output

```text
Morph Fast Context subagent performed search on repository:

Relevant context found:
- src/routes/middleware.ts:*
- src/core/router.ts:50-120

Here is the content of files:

<file path="src/routes/middleware.ts">
1| import { NextRequest } from 'next/server';
2| export function middleware(request: NextRequest) {
...
</file>

<file path="src/core/router.ts">
50| function resolveMiddleware(chain: MiddlewareChain) {
...
120| }
</file>
```

The output lists relevant files and their content from the GitHub repo. If a file path is wrong, you may see `Error: File not found` — retry with a more specific query.

## Rules

- **Write queries as natural language questions** — `"How does the router resolve middleware chains?"` works far better than `"router middleware"`, because the search agent plans its own strategy based on your question.
- **Be specific about what you want to know** — `"How does Prisma handle relation loading in findMany?"` beats `"Prisma relations"`.
- **Provide the owner/repo or URL** — the tool needs to know which GitHub repo to search.
- **Default timeout is 120s** — for large repos, increase with `--timeout 180` or higher.
