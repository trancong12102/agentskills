---
name: using-search-agent
description: Use when user asks about libraries, frameworks, APIs, current events, or any technical topic where up-to-date information is preferred over pretrained knowledge.
---

# Using the Search Agent

## Overview

Dispatch the `ccc:search` subagent for research tasks requiring current information. Prefer search over pretrained knowledge for documentation and implementation questions.

## When to Dispatch

**Always dispatch for:**

| Signal | Example |
|--------|---------|
| Library/framework questions | "How do I use tokio channels?" |
| API documentation | "What's the Stripe webhook signature format?" |
| Version-specific questions | "Next.js 15 middleware", "React 19 changes" |
| Time-sensitive terms | "latest", "recent", "2024", "2025", "current" |
| Best practices queries | "recommended way to...", "how should I..." |
| GitHub repo internals | "How does axum routing work?" |
| Troubleshooting with library | "Why is my Prisma query slow?" |

**Don't dispatch for:**

- Pure coding tasks (write function, fix bug in provided code)
- Codebase-specific questions (use Explore agent instead)
- Conceptual CS questions (algorithms, data structures)
- Questions already answered in current context

## Dispatch Pattern

```
Task tool:
  subagent_type: ccc:search
  prompt: [specific question + context]
  model: haiku (default, cost-effective)
```

## Prompt Tips

| Do | Don't |
|----|-------|
| Include version numbers | Ask vague questions |
| Specify language/framework | Omit relevant context |
| Ask focused questions | Bundle multiple topics |

**Good:** "How to handle streaming responses in Anthropic Python SDK v0.40+"
**Bad:** "Tell me about Anthropic SDK"

## Query Type → Tool Mapping

The search agent selects tools automatically, but understanding helps you craft better prompts:

| Query Type | Best Tool | Prompt Hint |
|------------|-----------|-------------|
| Library docs | context7 | Include library name |
| Current events/news | exa web search | Include year |
| Code examples | exa code context | Mention "example" or "code" |
| Repo internals | deepwiki | Include "owner/repo" format |

## Red Flags: When You Should Have Searched

- Answering from memory about library APIs
- Guessing at syntax or function signatures
- Saying "I believe..." about technical facts
- Version-specific questions without checking docs
