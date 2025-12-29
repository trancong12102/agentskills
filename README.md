# Agent Skills

A collection of reusable agent skills for AI coding assistants.

## Skills

| Name | Description |
|------|-------------|
| [conventional-commit](conventional-commit/SKILL.md) | Generates commit messages following Conventional Commits 1.0.0 specification |
| [openspec-proposal](openspec-proposal/SKILL.md) | Scaffold a new OpenSpec change and validate strictly |
| [openspec-apply](openspec-apply/SKILL.md) | Implement an approved OpenSpec change and keep tasks in sync |
| [openspec-archive](openspec-archive/SKILL.md) | Archive a deployed OpenSpec change and update specs |
| [searching-web](searching-web/SKILL.md) | Searches web, library documentation, GitHub repositories, and code examples |

## Environment Variables

Some skills require API keys. Add these to your shell configuration (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
# Required for searching-web skill
export CONTEXT7_API_KEY="your-context7-api-key"
export EXA_API_KEY="your-exa-api-key"
```

## Usage

### Ampcode

Install all skills:

```bash
amp skill add --global trancong12102/agentskills
```

Install a specific skill:

```bash
amp skill add --global trancong12102/agentskills/<skill-name>
```

### Manual

Copy the desired skill's `SKILL.md` file to your agent's skills directory.

## License

MIT
