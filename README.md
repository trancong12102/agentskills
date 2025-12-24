# CCC - Claude Code Core Skills

Core skills library for Claude Code: TDD, debugging, collaboration patterns, and proven techniques.

## Installation

### Quick Install

1. Open Claude Code in your terminal
2. Add this repository as a marketplace:

   ```bash
   /plugin marketplace add trancong12102/ccc
   ```

3. Install the plugin:

   ```bash
   /plugin install ccc@ccc-marketplace
   ```

### Alternative: Using Settings

Add to your project's `.claude/settings.json`:

```json
{
  "extraKnownMarketplaces": ["trancong12102/ccc"],
  "enabledPlugins": ["ccc@ccc-marketplace"]
}
```

### Verify Installation

```bash
/plugin list
```

You should see `ccc` in the list of installed plugins.

## Included Skills

### writing-skills

Use when creating new skills, editing existing skills, or verifying skills work before deployment.

This skill applies Test-Driven Development principles to process documentation - write test cases (pressure scenarios), watch them fail (baseline behavior), write the skill, and verify agents comply.

## Usage

Once installed, Claude Code will automatically discover and use the skills when relevant. Skills are loaded based on context matching - when Claude encounters a situation matching a skill's description, it will apply that skill's techniques.

## License

MIT
