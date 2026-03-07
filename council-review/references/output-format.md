# Council Review — Output Format

## Header

```markdown
## Council Review

**Verdict: <VERDICT>** · Reviewed by Codex + Claude

<1-2 sentence justification>
```

Verdict values:

- **Approved** — No issues or only informational notes
- **Approved with suggestions** — No critical/high issues, but improvements recommended
- **Request changes** — Critical or high-severity issues that should be fixed before merging

## Changes Walkthrough

| File              | Changes                           |
| ----------------- | --------------------------------- |
| `path/to/file.ts` | Brief description of what changed |

## Findings

All issues from all reviewers merged into a **single flat list**, deduplicated, sorted by severity (critical first). Each finding follows this format:

````markdown
#### <EMOJI> <Short title>

**<CATEGORY>** · `file/path.ts:LINE` · Confidence: <HIGH|MEDIUM>

Explanation of the issue and why it matters.

**Suggested fix:**
\```lang
code here
\```
````

**Severity emoji mapping:**

| Emoji | Severity | Criteria                                                     |
| ----- | -------- | ------------------------------------------------------------ |
| 🔴    | Critical | Exploitable vulnerability, data loss, or crash in production |
| 🟠    | High     | Likely bug or incident under realistic conditions            |
| 🟡    | Medium   | Incorrect behavior under edge cases or degraded performance  |
| 🟢    | Low      | Code quality issue that could escalate over time             |
| 🔵    | Info     | Observation or suggestion, no action required                |

**Categories:** `Bug`, `Security`, `Performance`, `Maintainability`, `Edge Case`, `Testing`, `Style`

**Confidence is derived from reviewer agreement (2 reviewers: Codex, Claude). Merge rules take precedence over these defaults when they specify a confidence level. Low-confidence findings (disputed by Claude or purely circumstantial) are excluded from the report — the council's value is cross-validation, and findings that fail it are noise.**

- **High** — Both reviewers flagged the same issue independently, or Claude confirmed a Codex finding
- **Medium** — One external reviewer flagged it and Claude did not dispute it, or Claude found it alone with clear evidence

If no issues found: "No issues found."

## Highlights

1-3 positive patterns worth calling out (good abstractions, solid error handling, thorough tests, etc.).
