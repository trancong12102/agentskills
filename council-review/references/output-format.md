# Council Review — Output Format

## Header

```markdown
## Council Review

**Verdict: <VERDICT>** · Reviewed by Codex + Gemini + Claude

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

**<CATEGORY>** · `file/path.ts:LINE` · Confidence: <HIGH|MEDIUM|LOW>

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

**Confidence is derived from reviewer agreement (3 reviewers: Codex, Gemini, Claude). Merge rules take precedence over these defaults when they specify a confidence level:**

- **High** — 2+ reviewers flagged the same issue independently, or Claude confirmed an external finding
- **Medium** — One external reviewer flagged it and Claude did not dispute it, or Claude found it alone with clear evidence
- **Low** — Evidence is circumstantial or only one external reviewer flagged it with weak justification

If no issues found: "No issues found."

## Highlights

1-3 positive patterns worth calling out (good abstractions, solid error handling, thorough tests, etc.).

## Raw Outputs

Always include at the end:

```markdown
<details>
<summary>Individual reviewer outputs</summary>

**Codex:**
<full codex output>

**Gemini:**
<full gemini output>

**Claude (/review):**
<Claude's /review output and validation notes>

</details>
```
