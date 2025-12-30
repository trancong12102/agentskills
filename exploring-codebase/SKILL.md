---
name: exploring-codebase
description: MUST USE for codebase exploration. Replaces finder, Grep, glob. Triggers on "where is", "how does", "find", "called", "used", or any code search question.
---

# Codebase Explorer

## When to Use Direct Calls vs Subagent

| Query Type | Approach |
|------------|----------|
| Simple (≤3 searches) | Call tools directly |
| Complex (>3 searches, multi-phase) | Spawn Task subagent |

## Simple Queries: Direct Tool Calls

For most code exploration, call tools directly:

1. **mcp__morph__warpgrep_codebase_search** — Natural language search (preferred)
2. **finder** — Semantic/conceptual search (fallback)
3. **Grep** — Exact string/pattern match (last resort)

```
Example: "where is typesense api called?"
→ Call warpgrep_codebase_search with search_string='where is typesense api called'
→ Return file paths with line numbers
```

## Complex Queries: Spawn Subagent

Use Task subagent when:
- Expecting many searches (>3-5 calls)
- Multi-file investigation or tracing flows
- Need structured analysis before reporting

```
Task(
  description: "<brief description>",
  prompt: "Investigate <complex question>.
Use warpgrep_codebase_search, then finder/Grep as needed.
Return: summary of findings with file paths and line numbers."
)
```

## Error Recovery

- No results → Broaden query, remove specific terms
- Wrong results → Rephrase with different terminology
