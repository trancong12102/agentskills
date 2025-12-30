---
name: exploring-codebase
description: Explores codebase using WarpGrep MCP for semantic code search. Use when finding code by behavior, concept, or functionality; when chaining multiple grep calls; when asking "where is X handled" or "how does Y work".
---

# WarpGrep Codebase Explorer

Performs semantic codebase search using `mcp__morph__warpgrep_codebase_search` instead of glob/grep.

## Quick Reference

| Need | Tool | When |
|------|------|------|
| Semantic/conceptual search | WarpGrep | "where is auth handled", "how does X work" |
| Find files by name pattern | glob | "find all *.test.ts files" |
| Exact string match | Grep | Known symbol or literal string |

## When to Use WarpGrep

- Finding code by behavior or concept
- Multi-step searches requiring chained grep calls
- Locating implementations across multiple files
- Understanding end-to-end feature flows
- Finding connections between codebase areas

## Tool Parameters

```json
{
  "search_string": "Where is JWT token validation performed",
  "repo_path": "/absolute/path/to/repo"
}
```

**Required:**
- `search_string`: Natural language query describing what to find
- `repo_path`: Absolute path to repository root

## Writing Effective Queries

| Goal | Good Query | Bad Query |
|------|-----------|-----------|
| Find auth | "Where is JWT token validation performed in the middleware" | "auth" |
| Locate handler | "Find the Express route handler for /api/users endpoint" | "users route" |
| Understand flow | "How does payment processing flow from API to database" | "payment" |
| Find integration | "Where does the frontend call the authentication API" | "login" |

**Tips:**
- Be specific: mention frameworks, patterns, file types
- Describe behavior, not just keywords
- Include architectural context when known

## Workflow

1. Use WarpGrep for conceptual/behavioral queries
2. Verify results with Read tool
3. Fall back to Grep for exact matches if needed

## Error Recovery

- No results → Broaden query, remove specific framework names
- Too many results → Add more context (file types, directory hints)
- Wrong results → Rephrase with different terminology
