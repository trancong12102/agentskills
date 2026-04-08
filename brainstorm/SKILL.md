---
name: brainstorm
description: "Structured requirements interview — asks targeted questions to reduce ambiguity before planning. Use when the user's request is vague, touches multiple systems, has unclear acceptance criteria, or could go multiple directions. Also use when the user says 'brainstorm', 'help me think through', 'what should I consider', or 'let's figure out'. Do NOT use for clear, well-scoped requests where you can go straight to planning."
---

# Brainstorm — Structured Requirements Interview

Reduce ambiguity to near-zero through targeted questions before entering plan mode. You are the user-facing interviewer — Metis handles codebase-facing analysis after you finish.

## When to Use

- Request is vague ("make it faster", "improve the UX")
- Scope touches multiple systems with unclear boundaries
- Acceptance criteria are implied, not stated
- Multiple valid approaches exist and user preference matters
- User explicitly asks to brainstorm or think through a problem

## When NOT to Use

- Request is clear and well-scoped ("add a logout button to the navbar")
- User already provided detailed requirements
- Trivial tasks (typo fixes, config changes)

## Workflow

### Phase 1 — Intent Classification

Classify the request into one of these types (same as Metis):

| Type          | Signal                                   | Interview Focus                           |
| ------------- | ---------------------------------------- | ----------------------------------------- |
| Refactoring   | "refactor", "restructure", "clean up"    | What behavior to preserve, what to change |
| Build         | "create", "add", "new feature"           | Scope boundaries, MVP vs full vision      |
| Mid-sized     | Scoped feature, specific deliverable     | Exact outputs, explicit exclusions        |
| Collaborative | "help me plan", "let's figure out"       | Open exploration, incremental clarity     |
| Architecture  | "how should we structure", system design | Constraints, scale, lifespan              |
| Research      | Investigation needed, path unclear       | Exit criteria, expected outputs           |

State the classification and confidence before proceeding.

### Phase 2 — Ambiguity Scoring

Score ambiguity across 4 dimensions (High / Medium / Low):

| Dimension      | High Ambiguity            | Low Ambiguity               |
| -------------- | ------------------------- | --------------------------- |
| **Scope**      | "improve performance"     | "optimize the /users query" |
| **Acceptance** | "should work well"        | "response time < 200ms"     |
| **Approach**   | multiple valid paths      | one obvious solution        |
| **Boundaries** | unclear what NOT to touch | explicit exclusions stated  |

Report the scores. Focus questions on the highest-ambiguity dimensions first.

### Phase 3 — Targeted Interview

Ask ONE question at a time using `AskUserQuestion`. Rules:

- **Most ambiguous dimension first** — attack the biggest unknown
- **Multi-choice when possible** — "Which approach: (A) Redis cache, (B) in-memory LRU, (C) CDN, (D) other?" is faster than open-ended
- **Concrete, not generic** — "Should auth middleware cover WebSocket routes too, or just REST?" beats "What's the scope?"
- **Build on previous answers** — each question should narrow the remaining ambiguity
- **State why you're asking** — "I'm asking because this determines whether we need a new database table or can reuse the existing one"

After each answer, mentally re-score the ambiguity dimensions. Continue until all dimensions score Low.

Do NOT ask more than 5 questions total. If ambiguity remains after 5, summarize what you know and what's still unclear — let the user decide whether to clarify further or proceed with assumptions.

### Phase 4 — Requirements Summary

Output a structured summary of what you learned:

```markdown
## Requirements Summary

**Intent**: [type] — [one-sentence description]

**Scope**:

- IN: [what's included]
- OUT: [what's explicitly excluded]

**Acceptance Criteria**:

1. [Concrete, verifiable criterion]
2. [Another criterion]

**Approach**: [chosen direction, if decided]

**Key Decisions**:

- [Decision]: [what user chose and why]

**Open Items** (if any):

- [remaining ambiguity that Metis/planning should resolve via codebase analysis]
```

After outputting the summary, tell the user they can now enter plan mode. Do NOT enter plan mode automatically — the user may want to refine the requirements further.

## Rules

- **Skill, not agent** — you run as the main agent. You CAN and SHOULD use `AskUserQuestion` for every question.
- **Do NOT explore the codebase** — that is Metis's job. You interview the user about intent, scope, and preferences.
- **Do NOT write files** — output the requirements summary in conversation. It feeds into plan mode naturally.
- **Do NOT propose solutions** — you gather requirements. Planning happens after.
- **5 question maximum** — respect the user's time. If you cannot reduce ambiguity in 5 questions, summarize and move on.
- **Match the user's language** — if they write in Vietnamese, interview in Vietnamese.
