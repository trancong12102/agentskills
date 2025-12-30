---
name: exploring-codebase
description: MUST USE for codebase exploration. Replaces finder, Grep, glob. Triggers on "where is", "how does", "find", "called", "used", or any code search question.
---

# Codebase Explorer

**Spawn a subagent using Task tool** to explore codebase. Do not call search tools directly.

## How to Use

When user asks a code exploration question, create a Task:

```
Task(
  description: "<brief description of what to find>",
  prompt: "Find <what user asked> in this codebase.

Use mcp__morph__warpgrep_codebase_search with:
- search_string: '<user question as natural language>'
- repo_path: '<workspace root absolute path>'

If warpgrep returns no results, use finder tool as fallback.
Only use Grep as last resort.

Return: file paths with line numbers, and brief summary of findings."
)
```

## Example

User: "where is typesense api called?"

```
Task(
  description: "Find where typesense API is called",
  prompt: "Find where typesense API is called in this codebase.

Use mcp__morph__warpgrep_codebase_search with:
- search_string: 'where is typesense api called'
- repo_path: <use actual workspace root>

If warpgrep returns no results, use finder tool as fallback.

Return: file paths with line numbers, and brief summary of findings."
)
```

## Why Subagent?

- Keeps main context clean (avoids pollution from search results)
- Subagent follows tool priority without built-in bias
- Returns only summarized findings

## Error Recovery

- No results → Broaden query, remove specific terms
- Wrong results → Rephrase with different terminology
- Warpgrep unavailable → Fall back to finder, then Grep
