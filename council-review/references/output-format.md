# Council Review — Output Format

## Header

```markdown
## Council Review

**Verdict: <VERDICT>** · Reviewed by Oracle + Claude + Simplify

<1-2 sentence justification>
```

Verdict values:

- **Approved** — No issues or only informational notes
- **Approved with suggestions** — No P0/P1 issues, but improvements recommended
- **Request changes** — P0 or P1 issues that should be fixed before merging

## Changes Walkthrough

| File              | Changes                           |
| ----------------- | --------------------------------- |
| `path/to/file.ts` | Brief description of what changed |

## Findings

All issues from all reviewers merged into a **single flat list**, deduplicated, sorted by priority (P0 first). Each finding follows this format:

````markdown
#### [<PRIORITY>] <Short title>

`<CATEGORY>` | `file/path.ts:LINE` | Confidence: <High|Medium>

Explanation of the issue and why it matters.

**Suggested fix:**
\```lang
code here
\```
````

**Priority levels:**

| Priority | Criteria                                                     |
| -------- | ------------------------------------------------------------ |
| P0       | Exploitable vulnerability, data loss, or crash in production |
| P1       | Likely bug or incident under realistic conditions            |
| P2       | Incorrect behavior under edge cases or degraded performance  |
| P3       | Code quality issue that could escalate over time             |
| P4       | Observation or suggestion, no action required                |

**Categories:** `Bug`, `Security`, `Performance`, `Maintainability`, `Edge Case`, `Testing`, `Style`

**Confidence is derived from reviewer agreement (3 reviewers: Oracle/Codex, Claude, Simplify). Merge rules take precedence over these defaults when they specify a confidence level. Low-confidence findings (disputed by Claude or purely circumstantial) are excluded from the report — the council's value is cross-validation, and findings that fail it are noise.**

- **High** — 2+ reviewers flagged the same issue independently, or Claude confirmed an external finding
- **Medium** — One external reviewer flagged it and Claude did not dispute it, or Claude found it alone with clear evidence

If no issues found: "No issues found."

## Highlights

1-3 positive patterns worth calling out (good abstractions, solid error handling, thorough tests, etc.).
