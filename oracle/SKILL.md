---
name: oracle
description: "Consult a second-opinion reasoning model for complex analysis, debugging, code review, or architectural decisions. Use when facing difficult bugs, reviewing critical code, designing refactors, or when the user says 'ask the oracle', 'use the oracle', 'second opinion', or 'consult oracle'."
---

# Oracle - Second Opinion Model

## Overview

The oracle is a powerful reasoning model optimized for complex analysis tasks. It excels at debugging, code review, architecture analysis, and finding better solutions - tasks requiring deep reasoning rather than quick edits.

**Trade-offs:** Slower and more expensive than the main agent, but significantly better at complex reasoning. Use deliberately, not for every task.

## When to Use the Oracle

**Good use cases:**

- Debugging complex or elusive bugs
- Reviewing critical code changes for correctness
- Analyzing architecture and suggesting improvements
- Refactoring planning with backward compatibility
- Finding better solutions to hard problems
- Verifying logic hasn't changed after modifications
- Understanding complex codebases or algorithms

**Poor use cases (use main agent instead):**

- Simple edits or typo fixes
- Straightforward feature implementation
- File operations and basic searches
- Tasks where speed matters more than depth

## How to Invoke

Use the Codex MCP tool with a focused prompt. The oracle works best with:

1. **Clear context**: Include relevant code, files, or error messages
2. **Specific question**: State exactly what you need analyzed
3. **Constraints**: Mention any requirements (backward compatibility, performance, etc.)

### Basic Invocation

```yaml
mcp__codex__codex with:
  prompt: "<your analysis request>"
  sandbox: "read-only"  # For analysis tasks
```

### For Tasks Requiring File Access

```yaml
mcp__codex__codex with:
  prompt: "<your request>"
  sandbox: "workspace-write"  # If changes needed
  approval-policy: "on-failure"
```

## Example Prompts

### Code Review

```text
Review the changes in the last commit. Verify that the actual logic for
[specific behavior] has not changed. Look for edge cases, race conditions,
and potential bugs.
```

### Debugging

```text
I have a bug in these files: [file list]
It shows up when: [reproduction steps]
Error: [error message]

Analyze the code flow and identify the root cause.
```

### Architecture Analysis

```text
Analyze how the functions `foo` and `bar` are used across the codebase.
Figure out how we can refactor the duplication between them while keeping
changes backward compatible.
```

### Finding Better Solutions

```text
I implemented [feature] using [approach]. Analyze whether there's a better
solution considering [constraints]. Compare trade-offs.
```

### Complex Refactoring

```text
Look at the [component] code. Design a refactoring plan that:
- Reduces code duplication
- Maintains backward compatibility
- Has clear separation of concerns
- Includes a migration path
```

## Workflow

1. **Gather context first**: Read relevant files, run commands, collect error logs
2. **Formulate a focused prompt**: Be specific about what you need analyzed
3. **Invoke the oracle**: Use Codex MCP with appropriate sandbox settings
4. **Act on the analysis**: Implement recommendations from the oracle's response

## Best Practices

- **Provide full context**: The oracle can't read your mind. Include code snippets, error messages, and constraints
- **Ask specific questions**: "Is this correct?" is worse than "Does this handle the edge case where X is null?"
- **Request actionable output**: Ask for specific recommendations, not just analysis
- **Use for verification**: After complex changes, ask the oracle to verify correctness
- **Chain with main agent**: Use oracle for analysis, main agent for implementation

## Integration Notes

The oracle uses the Codex MCP server (`mcp__codex__codex`). Key parameters:

| Parameter                | Description                                          | Default           |
| ------------------------ | ---------------------------------------------------- | ----------------- |
| `prompt`                 | The analysis request (required)                      | -                 |
| `sandbox`                | `read-only`, `workspace-write`, `danger-full-access` | `read-only`       |
| `approval-policy`        | `untrusted`, `on-failure`, `on-request`, `never`     | `untrusted`       |
| `cwd`                    | Working directory for the session                    | Current directory |
| `developer-instructions` | Additional context for the model                     | -                 |

For continued conversations with the oracle, use `mcp__codex__codex-reply` with the `threadId` from the initial response.
