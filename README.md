# skillinstaller

Install the same Agent Skill across many providers from a single `.skill/` source, with correct `project` vs `user` scope and deterministic path resolution.

> Library-first: embed the installer engine in your tooling.
> The CLI is a thin utility wrapper over the same engine.

One `.skill/` in, many providers out, with deterministic targets and deduplicated shared paths.

## What it solves

- Provider path sprawl: each provider uses different skills directory conventions.
- Scope confusion: `project` vs `user` installs should be explicit and predictable.
- Duplicate installs: providers sharing `.agents/skills` should resolve to one physical target.

## Primary Use: Embed as a Library

This project is intended to be consumed as a reusable installer engine by:

- internal platform CLIs
- setup/bootstrap tools
- CI/CD automation
- MCP/server-side skill orchestration

Core contract:

- `parseSkill(source) -> ParsedSkill`
- `supportedProviders() -> ProviderInfo[]`
- `detectProviders(projectRoot?) -> DetectedProvider[]`
- `promptProviderSelection(options?) -> InteractiveProviderSelectionResult`
- `resolveInstallTarget(provider, scope, projectRoot?) -> InstallTarget`
- `install(request) -> InstallResult`

Quick flow:

```text
skill = parseSkill(source)
providers = detectProviders(projectRoot)
result = install({ source, providers, scope, projectRoot, method, force })
```

Shared types:

- `Scope`: `project | user`
- `ProviderId`: provider slug (`claude-code`, `cursor`, `codex`, etc.)
- `SkillSource`: source containing `.skill/`
- `InstallRequest`: `{ source, providers, scope, projectRoot?, method, force? }`
- `InstallResult`: installed targets, normalized providers, warnings

Normalization rule:

- Providers using `.agents/skills` map to `universal` to avoid duplicate installs.
- Interactive selection helper keeps `.agents` providers locked/included via `universal` and only lets users select non-universal providers.

## CLI (Utility)

```bash
cargo run --bin install-skill -- install \
  --source ./path/to/skill-repo \
  --providers claude-code,cursor \
  --scope project \
  --project-root /path/to/project
```

### Source Format

The source must contain:

```text
.skill/
  SKILL.md
  ...any extra files/folders
```

`SKILL.md` is parsed for frontmatter (`name` required), and the full `.skill/*` payload is installed.

### Options

| Option | Description |
| --- | --- |
| `--source <path>` | Path containing `.skill/` (or direct `.skill` path) |
| `--providers <list|'*'>` | Comma-separated providers (`claude-code,cursor`) or `'*'` for all |
| `--scope <project|user>` | Installation scope |
| `--project-root <path>` | Required when `--scope project` |
| `--method <symlink|copy>` | Installation method |
| `--force` | Overwrite existing installed skill directory |

### Examples

```bash
# List supported providers
cargo run --bin install-skill -- providers

# Detect installed providers from machine/project signals
cargo run --bin install-skill -- detect --project-root /path/to/project

# Install to user scope for all providers
cargo run --bin install-skill -- install \
  --source ./my-skill-source \
  --providers '*' \
  --scope user \
  --method symlink \
  --force
```

### Installation Scope

| Scope | Location | Use Case |
| --- | --- | --- |
| `project` | `<project_root>/<provider skills path>/` | Share via repo and team workflows |
| `user` | Provider global config dir in home | Available across local projects |

## Provider behavior

- Providers that use `.agents/skills` are normalized to `universal`.
- This avoids duplicate installs when multiple selected providers share the same physical path.
- Non-`.agents` providers are installed to provider-specific paths.

## Supported Providers

| Provider | `--providers` | Project Path | Global Path |
| --- | --- | --- | --- |
| Amp | `amp` | `.agents/skills/` | `~/.config/agents/skills/` |
| Antigravity | `antigravity` | `.agent/skills/` | `~/.gemini/antigravity/skills/` |
| Augment | `augment` | `.augment/skills/` | `~/.augment/skills/` |
| Claude Code | `claude-code` | `.claude/skills/` | `~/.claude/skills/` |
| OpenClaw | `openclaw` | `skills/` | `~/.openclaw/skills/` |
| Cline | `cline` | `.agents/skills/` | `~/.agents/skills/` |
| CodeBuddy | `codebuddy` | `.codebuddy/skills/` | `~/.codebuddy/skills/` |
| Codex | `codex` | `.agents/skills/` | `~/.codex/skills/` |
| Command Code | `command-code` | `.commandcode/skills/` | `~/.commandcode/skills/` |
| Continue | `continue` | `.continue/skills/` | `~/.continue/skills/` |
| Cortex Code | `cortex` | `.cortex/skills/` | `~/.snowflake/cortex/skills/` |
| Crush | `crush` | `.crush/skills/` | `~/.config/crush/skills/` |
| Cursor | `cursor` | `.agents/skills/` | `~/.cursor/skills/` |
| Droid | `droid` | `.factory/skills/` | `~/.factory/skills/` |
| Gemini CLI | `gemini-cli` | `.agents/skills/` | `~/.gemini/skills/` |
| GitHub Copilot | `github-copilot` | `.agents/skills/` | `~/.copilot/skills/` |
| Goose | `goose` | `.goose/skills/` | `~/.config/goose/skills/` |
| Junie | `junie` | `.junie/skills/` | `~/.junie/skills/` |
| iFlow CLI | `iflow-cli` | `.iflow/skills/` | `~/.iflow/skills/` |
| Kilo Code | `kilo` | `.kilocode/skills/` | `~/.kilocode/skills/` |
| Kimi Code CLI | `kimi-cli` | `.agents/skills/` | `~/.config/agents/skills/` |
| Kiro CLI | `kiro-cli` | `.kiro/skills/` | `~/.kiro/skills/` |
| Kode | `kode` | `.kode/skills/` | `~/.kode/skills/` |
| MCPJam | `mcpjam` | `.mcpjam/skills/` | `~/.mcpjam/skills/` |
| Mistral Vibe | `mistral-vibe` | `.vibe/skills/` | `~/.vibe/skills/` |
| Mux | `mux` | `.mux/skills/` | `~/.mux/skills/` |
| OpenCode | `opencode` | `.agents/skills/` | `~/.config/opencode/skills/` |
| OpenHands | `openhands` | `.openhands/skills/` | `~/.openhands/skills/` |
| Pi | `pi` | `.pi/skills/` | `~/.pi/agent/skills/` |
| Qoder | `qoder` | `.qoder/skills/` | `~/.qoder/skills/` |
| Qwen Code | `qwen-code` | `.qwen/skills/` | `~/.qwen/skills/` |
| Replit | `replit` | `.agents/skills/` | `~/.config/agents/skills/` |
| Roo Code | `roo` | `.roo/skills/` | `~/.roo/skills/` |
| Trae | `trae` | `.trae/skills/` | `~/.trae/skills/` |
| Trae CN | `trae-cn` | `.trae/skills/` | `~/.trae-cn/skills/` |
| Windsurf | `windsurf` | `.windsurf/skills/` | `~/.codeium/windsurf/skills/` |
| Zencoder | `zencoder` | `.zencoder/skills/` | `~/.zencoder/skills/` |
| Neovate | `neovate` | `.neovate/skills/` | `~/.neovate/skills/` |
| Pochi | `pochi` | `.pochi/skills/` | `~/.pochi/skills/` |
| AdaL | `adal` | `.adal/skills/` | `~/.adal/skills/` |
| Universal (shared target) | `universal` | `.agents/skills/` | `~/.config/agents/skills/` |

## Commands

| Command | Description |
| --- | --- |
| `install-skill providers` | List supported providers |
| `install-skill detect` | Detect providers on current machine |
| `install-skill install` | Install a `.skill` payload |

## What are Agent Skills?

Agent Skills are reusable instruction packages defined by `SKILL.md` + optional supporting files.

- Spec reference: https://agentskills.io/specification
- Provider path model inspiration: https://github.com/vercel-labs/skills

## Notes

- This README documents the installer contract, independent of implementation language.
- The CLI exists for local testing and operational convenience.
- Interactive prompts are available behind the `interactive` feature flag in Rust (`features = ["interactive"]`).
