# Council Review — Merge Rules

When synthesizing findings from all three reviewers (Codex, Gemini, Claude):

1. **Same issue, same fix, Claude confirmed** → Merge into one finding, confidence: High
2. **Same issue, different fix** → Merge into one finding, confidence: High, present the best fix (prefer Claude's improved version if available)
3. **External finding confirmed by Claude** → Include with Claude's enhanced explanation if applicable, confidence: High
4. **External finding disputed by Claude** → Include the finding, confidence: Low, note Claude's reasoning for the dispute
5. **Contradictory assessments between external reviewers, Claude breaks the tie** → Include with Claude's assessment as the deciding factor, confidence: Medium
6. **Unique finding from one external reviewer, not disputed by Claude** → Include as-is, confidence: Medium
7. **Unique finding from Claude only** → Include as Claude's own finding, confidence: Medium
