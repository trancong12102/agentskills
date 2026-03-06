# Council Oracle — Synthesis Rules

When synthesizing findings from both oracles (Codex, Claude subagent):

1. **Same conclusion from both oracles** — Merge into one finding, confidence: High
2. **Same conclusion, different reasoning** — Merge into one finding, confidence: High, present the best reasoning (prefer the most evidence-backed explanation)
3. **Unique insight from one oracle, not contradicted by others** — Include as-is, confidence: Medium
4. **Contradictory conclusions between oracles** — Claude breaks the tie using its own analysis and codebase knowledge, confidence: Medium, note the disagreement
5. **Vague or generic advice** — Drop entirely. Only include findings grounded in specific codebase evidence
6. **Hallucinated file paths or code references** — Verify against the actual codebase before including. Drop if unverifiable
7. **Same recommendation, different priority levels** — Use the highest priority from any oracle, unless Claude's analysis justifies lowering it
