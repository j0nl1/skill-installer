# skill-installer (TypeScript)

Library-first installer engine for Agent Skills.

- Parse `.skill/SKILL.md`
- Detect providers
- Resolve install targets (`project` or `user`)
- Install full `.skill/*` payload
- Normalize shared `.agents/skills` providers to `universal`

## API

- `parseSkill(source)`
- `supportedProviders()`
- `detectProviders(projectRoot?)`
- `promptProviderSelection(options?)`
- `resolveInstallTarget(provider, scope, projectRoot?)`
- `install(request)`
