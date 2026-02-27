# skill-installer (TypeScript)

Library-first installer engine for Agent Skills.

- Parse `.skill/SKILL.md`
- Detect providers
- Resolve install targets (`project` or `user`)
- Install full `.skill/*` payload
- Normalize shared `.agents/skills` providers to `universal`

## Integration Case Studies

Real integrations:

- `steam-cli`: https://github.com/j0nl1/steam-cli/commit/3a6838faf2fb2db08d069547b7956a19c44e00dc
- `aitracker`: https://github.com/j0nl1/aitracker/commit/ec348cf0d6d9155c98f3f51912dc80ac6f20930f

## API

- `parseSkill(source)`
- `supportedProviders()`
- `detectProviders(projectRoot?)`
- `promptProviderSelection(options?)`
- `resolveInstallTarget(provider, scope, projectRoot?)`
- `install(request)`
