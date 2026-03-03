# Council Oracle — Output Format

## Header

```markdown
## Oracle Analysis

**Analyzed by Gemini + Codex + Claude**

<1-2 sentence summary of the analysis and key conclusion>
```

## Analysis

Organize findings by **theme or topic**, not by oracle source. The structure adapts to the question type:

- **Architecture analysis** — organize by component, layer, or concern (coupling, cohesion, extensibility)
- **Bug debugging** — organize by root cause hypothesis, ranked by likelihood
- **Security reasoning** — organize by threat category (injection, auth, data exposure, etc.)
- **Refactoring strategy** — organize by refactoring dimension (structure, naming, abstraction, dependencies)
- **Impact assessment** — organize by affected area (code, tests, performance, users, deployment)
- **Trade-off evaluation** — organize by option, with pros/cons for each

Each finding includes a confidence tag:

```markdown
### <Theme Title>

**Confidence: High | Medium | Low**

<Detailed analysis with specific file paths and line references>
```

## Recommendations

Prioritized numbered list. Each recommendation includes priority level and rationale:

```markdown
## Recommendations

1. **[Critical]** <Action> — <Rationale>
2. **[High]** <Action> — <Rationale>
3. **[Medium]** <Action> — <Rationale>
4. **[Low]** <Action> — <Rationale>
```

Priority definitions:

- **Critical** — Do this now; blocking issue or significant risk
- **High** — Do this soon; important for correctness or quality
- **Medium** — Do this eventually; improves maintainability or performance
- **Low** — Nice to have; minor improvement

## Risks and Caveats

Known unknowns, assumptions made during analysis, and areas where further investigation is needed:

```markdown
## Risks and Caveats

- <Risk or caveat with likelihood and suggested mitigation>
```
