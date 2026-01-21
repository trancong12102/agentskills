---
name: oracle
description: "This skill should be used when the user asks to 'ask the oracle', 'use the oracle', 'get a second opinion', 'consult oracle', 'deep analysis', or when facing difficult bugs, reviewing critical code, designing complex refactors, or needing architectural analysis. Invokes a powerful reasoning model via Codex MCP for complex analysis tasks."
---

# Oracle - Second Opinion Model

## Overview

The oracle invokes a powerful reasoning model (OpenAI's gpt-5-codex/o3) via Codex MCP, optimized for complex analysis tasks. It excels at debugging, code review, architecture analysis, and finding better solutions.

**Trade-offs:** Slower and more expensive than the main agent, but significantly better at complex reasoning. Use deliberately, not for every task.

## When to Use

**Good use cases:**

- Debugging complex or elusive bugs
- Reviewing critical code changes for correctness
- Analyzing architecture and suggesting improvements
- Refactoring planning with backward compatibility
- Finding better solutions to hard problems
- Verifying logic hasn't changed after modifications
- Understanding complex codebases or algorithms
- Researching unfamiliar APIs or libraries (with web search)

**Poor use cases (use main agent instead):**

- Simple edits or typo fixes
- Straightforward feature implementation
- File operations and basic searches
- Tasks where speed matters more than depth

## Codex Capabilities

Codex has access to powerful tools that enhance the oracle's analysis:

### File Reading

Include files directly in prompts using the `@` syntax:

```yaml
mcp__codex__codex with:
  prompt: "@src/auth/login.ts @src/auth/session.ts Analyze the authentication flow and identify security issues"
```

### Web Search

Enable web search to let Codex research documentation, best practices, and solutions:

```yaml
mcp__codex__codex with:
  prompt: "Research the best practices for implementing rate limiting in Express.js, then analyze our current implementation in @src/middleware/rateLimit.ts"
  developer-instructions: "Use web search to find current best practices before analyzing the code"
```

### Combined Research and Analysis

For comprehensive analysis, instruct Codex to:

1. Search the web for relevant documentation/patterns
2. Read the relevant files from the codebase
3. Provide analysis with recommendations

```yaml
mcp__codex__codex with:
  prompt: |
    Task: Analyze our Redis caching implementation for issues.

    Steps:
    1. Search for current Redis caching best practices and common pitfalls
    2. Read our implementation in @src/cache/*.ts
    3. Compare our implementation against best practices
    4. Provide specific recommendations with code examples
  sandbox: "read-only"
```

## Invocation Patterns

### Basic Analysis (Read-Only)

```yaml
mcp__codex__codex with:
  prompt: "<analysis request>"
  sandbox: "read-only"
```

### Analysis with File References

```yaml
mcp__codex__codex with:
  prompt: "@file1.ts @file2.ts <analysis request>"
  sandbox: "read-only"
```

### Research-Enhanced Analysis

```yaml
mcp__codex__codex with:
  prompt: |
    Research [topic] using web search, then analyze:
    @relevant/files.ts

    Provide recommendations based on current best practices.
  sandbox: "read-only"
```

### Tasks Requiring File Modifications

```yaml
mcp__codex__codex with:
  prompt: "<request>"
  sandbox: "workspace-write"
  approval-policy: "on-failure"
```

## Example Prompts

### Code Review with Context Research

```text
Search for common security vulnerabilities in JWT implementations.
Then review @src/auth/jwt.ts for these issues.
Provide specific fixes for any vulnerabilities found.
```

### Debugging with Documentation Lookup

```text
Research the expected behavior of React's useEffect cleanup function.
Then analyze @src/components/DataFetcher.tsx to find why the memory leak occurs.
The component fetches data but doesn't clean up properly on unmount.
```

### Architecture Analysis

```text
Analyze how @src/services/payment.ts and @src/services/order.ts interact.
Search for best practices on separating payment and order concerns.
Propose a refactoring plan that maintains backward compatibility.
```

### Finding Better Solutions

```text
I implemented caching using @src/cache/redis.ts
Search for modern Redis caching patterns (cache-aside, write-through, etc.)
Analyze whether there's a better approach considering:
- High read volume
- Occasional cache stampedes
- Need for cache invalidation on writes
```

### Complex Bug Investigation

```text
Bug: Users intermittently see stale data after updates.
Related files: @src/api/update.ts @src/cache/invalidation.ts @src/hooks/useData.ts

1. Search for common causes of stale data in React + Redis architectures
2. Trace the data flow through these files
3. Identify race conditions or cache invalidation issues
4. Provide a fix with explanation
```

## Workflow

1. **Gather context first**: Identify relevant files and the specific problem
2. **Formulate a focused prompt**: Include file references with `@`, specify research needs
3. **Invoke the oracle**: Use Codex MCP with appropriate sandbox settings
4. **Act on the analysis**: Implement recommendations from the oracle's response

## Best Practices

- **Use `@` syntax for files**: Include relevant files directly in the prompt
- **Request web research**: Ask Codex to search for best practices, documentation, or solutions
- **Provide full context**: Include error messages, reproduction steps, and constraints
- **Ask specific questions**: "Does this handle null edge cases?" beats "Is this correct?"
- **Request actionable output**: Ask for specific recommendations, not just analysis
- **Chain with main agent**: Use oracle for analysis/research, main agent for implementation

## Integration Reference

| Parameter                | Description                                          | Default           |
| ------------------------ | ---------------------------------------------------- | ----------------- |
| `prompt`                 | The analysis request (required). Use `@file` syntax  | -                 |
| `sandbox`                | `read-only`, `workspace-write`, `danger-full-access` | `read-only`       |
| `approval-policy`        | `untrusted`, `on-failure`, `on-request`, `never`     | `untrusted`       |
| `cwd`                    | Working directory for the session                    | Current directory |
| `developer-instructions` | Additional context/instructions for the model        | -                 |

For continued conversations, use `mcp__codex__codex-reply` with the `threadId` from the initial response.
