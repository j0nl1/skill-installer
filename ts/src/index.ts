import { cp, mkdir, readFile, rm, stat, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { createInterface } from 'node:readline/promises';

export type Scope = 'user' | 'project';

export type ProviderId =
  | 'amp'
  | 'antigravity'
  | 'augment'
  | 'claude-code'
  | 'openclaw'
  | 'cline'
  | 'codebuddy'
  | 'codex'
  | 'command-code'
  | 'continue'
  | 'cortex'
  | 'crush'
  | 'cursor'
  | 'droid'
  | 'gemini-cli'
  | 'github-copilot'
  | 'goose'
  | 'junie'
  | 'iflow-cli'
  | 'kilo'
  | 'kimi-cli'
  | 'kiro-cli'
  | 'kode'
  | 'mcpjam'
  | 'mistral-vibe'
  | 'mux'
  | 'opencode'
  | 'openhands'
  | 'pi'
  | 'qoder'
  | 'qwen-code'
  | 'replit'
  | 'roo'
  | 'trae'
  | 'trae-cn'
  | 'windsurf'
  | 'zencoder'
  | 'neovate'
  | 'pochi'
  | 'adal'
  | 'universal';

export interface ProviderInfo {
  id: ProviderId;
  displayName: string;
  usesAgentsDir: boolean;
  projectPath: string;
}

export interface ParsedSkill {
  name: string;
  description?: string;
  metadata?: Record<string, string>;
  allowedTools?: string;
  body: string;
}

export type SkillSource =
  | { type: 'local'; path: string }
  | { type: 'embedded'; skillMd: string; files: Array<{ relativePath: string; content: Uint8Array }> };

export interface InstallRequest {
  source: SkillSource;
  providers: ProviderId[];
  scope: Scope;
  projectRoot?: string;
  force?: boolean;
}

export interface InstallTarget {
  requestedProvider: ProviderId;
  targetProvider: ProviderId;
  targetDir: string;
}

export interface InstallResult {
  skillName: string;
  installedTargets: InstallTarget[];
  normalizedProviders: Array<{ from: ProviderId; to: ProviderId }>;
  skippedDuplicates: string[];
  warnings: string[];
}

export interface InteractiveProviderSelectionOptions {
  candidateProviders?: ProviderId[];
  defaultSelected?: ProviderId[];
  message?: string;
  input?: NodeJS.ReadableStream;
  output?: NodeJS.WritableStream;
}

export interface InteractiveProviderSelectionResult {
  universalProviders: ProviderId[];
  selectableProviders: ProviderId[];
  selectedProviders: ProviderId[];
}

const providers: ProviderInfo[] = [
  { id: 'amp', displayName: 'Amp', usesAgentsDir: true, projectPath: '.agents/skills' },
  { id: 'antigravity', displayName: 'Antigravity', usesAgentsDir: false, projectPath: '.agent/skills' },
  { id: 'augment', displayName: 'Augment', usesAgentsDir: false, projectPath: '.augment/skills' },
  { id: 'claude-code', displayName: 'Claude Code', usesAgentsDir: false, projectPath: '.claude/skills' },
  { id: 'openclaw', displayName: 'OpenClaw', usesAgentsDir: false, projectPath: 'skills' },
  { id: 'cline', displayName: 'Cline', usesAgentsDir: true, projectPath: '.agents/skills' },
  { id: 'codebuddy', displayName: 'CodeBuddy', usesAgentsDir: false, projectPath: '.codebuddy/skills' },
  { id: 'codex', displayName: 'Codex', usesAgentsDir: true, projectPath: '.agents/skills' },
  { id: 'command-code', displayName: 'Command Code', usesAgentsDir: false, projectPath: '.commandcode/skills' },
  { id: 'continue', displayName: 'Continue', usesAgentsDir: false, projectPath: '.continue/skills' },
  { id: 'cortex', displayName: 'Cortex Code', usesAgentsDir: false, projectPath: '.cortex/skills' },
  { id: 'crush', displayName: 'Crush', usesAgentsDir: false, projectPath: '.crush/skills' },
  { id: 'cursor', displayName: 'Cursor', usesAgentsDir: true, projectPath: '.agents/skills' },
  { id: 'droid', displayName: 'Droid', usesAgentsDir: false, projectPath: '.factory/skills' },
  { id: 'gemini-cli', displayName: 'Gemini CLI', usesAgentsDir: true, projectPath: '.agents/skills' },
  { id: 'github-copilot', displayName: 'GitHub Copilot', usesAgentsDir: true, projectPath: '.agents/skills' },
  { id: 'goose', displayName: 'Goose', usesAgentsDir: false, projectPath: '.goose/skills' },
  { id: 'junie', displayName: 'Junie', usesAgentsDir: false, projectPath: '.junie/skills' },
  { id: 'iflow-cli', displayName: 'iFlow CLI', usesAgentsDir: false, projectPath: '.iflow/skills' },
  { id: 'kilo', displayName: 'Kilo Code', usesAgentsDir: false, projectPath: '.kilocode/skills' },
  { id: 'kimi-cli', displayName: 'Kimi Code CLI', usesAgentsDir: true, projectPath: '.agents/skills' },
  { id: 'kiro-cli', displayName: 'Kiro CLI', usesAgentsDir: false, projectPath: '.kiro/skills' },
  { id: 'kode', displayName: 'Kode', usesAgentsDir: false, projectPath: '.kode/skills' },
  { id: 'mcpjam', displayName: 'MCPJam', usesAgentsDir: false, projectPath: '.mcpjam/skills' },
  { id: 'mistral-vibe', displayName: 'Mistral Vibe', usesAgentsDir: false, projectPath: '.vibe/skills' },
  { id: 'mux', displayName: 'Mux', usesAgentsDir: false, projectPath: '.mux/skills' },
  { id: 'opencode', displayName: 'OpenCode', usesAgentsDir: true, projectPath: '.agents/skills' },
  { id: 'openhands', displayName: 'OpenHands', usesAgentsDir: false, projectPath: '.openhands/skills' },
  { id: 'pi', displayName: 'Pi', usesAgentsDir: false, projectPath: '.pi/skills' },
  { id: 'qoder', displayName: 'Qoder', usesAgentsDir: false, projectPath: '.qoder/skills' },
  { id: 'qwen-code', displayName: 'Qwen Code', usesAgentsDir: false, projectPath: '.qwen/skills' },
  { id: 'replit', displayName: 'Replit', usesAgentsDir: true, projectPath: '.agents/skills' },
  { id: 'roo', displayName: 'Roo Code', usesAgentsDir: false, projectPath: '.roo/skills' },
  { id: 'trae', displayName: 'Trae', usesAgentsDir: false, projectPath: '.trae/skills' },
  { id: 'trae-cn', displayName: 'Trae CN', usesAgentsDir: false, projectPath: '.trae/skills' },
  { id: 'windsurf', displayName: 'Windsurf', usesAgentsDir: false, projectPath: '.windsurf/skills' },
  { id: 'zencoder', displayName: 'Zencoder', usesAgentsDir: false, projectPath: '.zencoder/skills' },
  { id: 'neovate', displayName: 'Neovate', usesAgentsDir: false, projectPath: '.neovate/skills' },
  { id: 'pochi', displayName: 'Pochi', usesAgentsDir: false, projectPath: '.pochi/skills' },
  { id: 'adal', displayName: 'AdaL', usesAgentsDir: false, projectPath: '.adal/skills' },
  { id: 'universal', displayName: 'Universal', usesAgentsDir: true, projectPath: '.agents/skills' }
];

export function supportedProviders(): ProviderInfo[] {
  return [...providers];
}

export function isAgentsProvider(provider: ProviderId): boolean {
  return providers.find((p) => p.id === provider)?.usesAgentsDir ?? false;
}

export function normalizeProviders(ids: ProviderId[]): {
  normalized: ProviderId[];
  mappings: Array<{ from: ProviderId; to: ProviderId }>;
} {
  const normalized: ProviderId[] = [];
  const mappings: Array<{ from: ProviderId; to: ProviderId }> = [];
  const seen = new Set<ProviderId>();

  for (const id of ids) {
    const target: ProviderId = isAgentsProvider(id) ? 'universal' : id;
    if (target !== id) mappings.push({ from: id, to: target });
    if (!seen.has(target)) {
      seen.add(target);
      normalized.push(target);
    }
  }

  return { normalized, mappings };
}

export async function parseSkill(source: SkillSource): Promise<ParsedSkill> {
  const content =
    source.type === 'embedded' ? source.skillMd : await readFile(join(await localSkillRoot(source.path), 'SKILL.md'), 'utf-8');

  if (!content.startsWith('---\n')) throw new Error('InvalidFrontmatter: missing opening frontmatter delimiter');
  const rest = content.slice(4);
  const end = rest.indexOf('\n---\n');
  if (end < 0) throw new Error('InvalidFrontmatter: missing closing frontmatter delimiter');

  const yamlRaw = rest.slice(0, end);
  const body = rest.slice(end + 5);

  const lines = yamlRaw.split('\n');
  const map: Record<string, string> = {};
  let inMetadata = false;
  const metadata: Record<string, string> = {};
  for (const line of lines) {
    if (line.startsWith('metadata:')) {
      inMetadata = true;
      continue;
    }
    if (inMetadata && line.startsWith('  ') && line.includes(':')) {
      const [k, ...v] = line.trim().split(':');
      metadata[k.trim()] = v.join(':').trim().replace(/^"|"$/g, '');
      continue;
    }
    inMetadata = false;
    if (line.includes(':')) {
      const [k, ...v] = line.split(':');
      map[k.trim()] = v.join(':').trim().replace(/^"|"$/g, '');
    }
  }

  if (!map.name) throw new Error('MissingName');
  if (/[/\\:*?"<>|]/.test(map.name) || map.name === '.' || map.name === '..') throw new Error(`InvalidName: ${map.name}`);

  return {
    name: map.name,
    description: map.description || undefined,
    allowedTools: map['allowed-tools'] || undefined,
    metadata: Object.keys(metadata).length ? metadata : undefined,
    body
  };
}

export async function detectProviders(projectRoot?: string): Promise<Array<{ provider: ProviderId; reason: string }>> {
  const home = homedir();
  const out: Array<{ provider: ProviderId; reason: string }> = [];

  for (const provider of providers) {
    if (provider.id === 'universal') continue;

    const marker = markerPath(provider.id, home, projectRoot);
    if (marker && existsSync(marker)) {
      out.push({ provider: provider.id, reason: `found ${marker}` });
      continue;
    }

    if (projectRoot) {
      const projectPath = join(projectRoot, provider.projectPath);
      if (existsSync(projectPath)) {
        out.push({ provider: provider.id, reason: `found ${projectPath}` });
      }
    }
  }

  return out;
}

export async function promptProviderSelection(
  options: InteractiveProviderSelectionOptions = {}
): Promise<InteractiveProviderSelectionResult> {
  const input = options.input ?? process.stdin;
  const output = options.output ?? process.stdout;
  const message = options.message ?? 'Select providers to install to';

  const candidateProviders = Array.from(
    new Set(
      (options.candidateProviders ?? providers.map((p) => p.id))
        .filter((p): p is ProviderId => providers.some((known) => known.id === p))
        .filter((p) => p !== 'universal')
    )
  );

  const universalProviders: ProviderId[] = [];
  const selectableProviders: ProviderId[] = [];
  for (const provider of candidateProviders) {
    if (isAgentsProvider(provider)) {
      universalProviders.push(provider);
    } else {
      selectableProviders.push(provider);
    }
  }
  const selectableSet = new Set<ProviderId>(selectableProviders);
  const defaultSelected = Array.from(
    new Set(
      (options.defaultSelected ?? selectableProviders).filter((p): p is ProviderId =>
        selectableSet.has(p as ProviderId)
      )
    )
  );

  if (selectableProviders.length === 0) {
    return {
      universalProviders,
      selectableProviders,
      selectedProviders: universalProviders.length > 0 ? ['universal'] : []
    };
  }

  const rl = createInterface({ input, output });
  try {
    output.write(`${message}\n`);
    if (universalProviders.length > 0) {
      const lockedLabels = universalProviders.map((p) => `${providerLabel(p)} (${p})`).join(', ');
      output.write(`Universal (.agents/skills) [always included]: ${lockedLabels}\n`);
    }

    output.write('Additional providers:\n');
    selectableProviders.forEach((provider, index) => {
      output.write(`  ${index + 1}. ${providerLabel(provider)} (${provider})\n`);
    });

    const defaultText = defaultSelected.length > 0 ? defaultSelected.join(',') : 'none';
    output.write(
      `Enter comma-separated numbers or provider ids ('*' for all, Enter for default: ${defaultText})\n`
    );

    while (true) {
      const answer = (await rl.question('> ')).trim();
      let selected: ProviderId[] = [];

      if (answer === '') {
        selected = [...defaultSelected];
      } else if (answer === '*') {
        selected = [...selectableProviders];
      } else {
        const tokens = answer
          .split(',')
          .map((t) => t.trim())
          .filter(Boolean);
        const parsed = new Set<ProviderId>();
        let invalid = false;

        for (const token of tokens) {
          const maybeIndex = Number(token);
          if (Number.isInteger(maybeIndex) && maybeIndex >= 1 && maybeIndex <= selectableProviders.length) {
            parsed.add(selectableProviders[maybeIndex - 1]!);
            continue;
          }
          const providerToken = token as ProviderId;
          if (selectableSet.has(providerToken)) {
            parsed.add(providerToken);
            continue;
          }
          invalid = true;
          break;
        }

        if (invalid) {
          output.write('Invalid selection. Use listed numbers, provider ids, * or Enter.\n');
          continue;
        }

        selected = [...parsed];
      }

      return {
        universalProviders,
        selectableProviders,
        selectedProviders: [
          ...(universalProviders.length > 0 ? (['universal'] as ProviderId[]) : []),
          ...selected
        ]
      };
    }
  } finally {
    rl.close();
  }
}

export function resolveInstallTarget(
  requestedProvider: ProviderId,
  scope: Scope,
  projectRoot?: string
): InstallTarget {
  const targetProvider: ProviderId = isAgentsProvider(requestedProvider) ? 'universal' : requestedProvider;
  const base = resolveProviderDir(targetProvider, scope, projectRoot);
  return { requestedProvider, targetProvider, targetDir: base };
}

export async function install(request: InstallRequest): Promise<InstallResult> {
  const parsed = await parseSkill(request.source);
  const { normalized, mappings } = normalizeProviders(request.providers);

  const installedTargets: InstallTarget[] = [];
  const skippedDuplicates: string[] = [];
  const warnings: string[] = [];
  const seen = new Set<string>();

  for (const provider of normalized) {
    const resolved = resolveInstallTarget(provider, request.scope, request.projectRoot);
    const destination = join(resolved.targetDir, parsed.name);

    if (seen.has(destination)) {
      skippedDuplicates.push(destination);
      continue;
    }
    seen.add(destination);

    if (!request.force && existsSync(destination)) {
      throw new Error(`AlreadyExists: ${destination}`);
    }

    await copySourceToDestination(request.source, destination);

    installedTargets.push({
      requestedProvider: provider,
      targetProvider: resolved.targetProvider,
      targetDir: destination
    });
  }

  for (const m of mappings) {
    warnings.push(`provider '${m.from}' normalized to '${m.to}' shared .agents target`);
  }

  return {
    skillName: parsed.name,
    installedTargets,
    normalizedProviders: mappings,
    skippedDuplicates,
    warnings
  };
}

function resolveProviderDir(provider: ProviderId, scope: Scope, projectRoot?: string): string {
  const home = homedir();
  const configHome = process.env.XDG_CONFIG_HOME || join(home, '.config');

  if (scope === 'project') {
    if (!projectRoot) throw new Error('ProjectRootRequired');
    const info = providers.find((p) => p.id === provider);
    if (!info) throw new Error(`UnsupportedProvider: ${provider}`);
    return join(projectRoot, info.projectPath);
  }

  switch (provider) {
    case 'universal':
    case 'amp':
    case 'kimi-cli':
    case 'replit':
      return join(configHome, 'agents/skills');
    case 'codex':
      return join(process.env.CODEX_HOME || join(home, '.codex'), 'skills');
    case 'claude-code':
      return join(process.env.CLAUDE_CONFIG_DIR || join(home, '.claude'), 'skills');
    case 'openclaw':
      return existsSync(join(home, '.openclaw'))
        ? join(home, '.openclaw/skills')
        : existsSync(join(home, '.clawdbot'))
          ? join(home, '.clawdbot/skills')
          : existsSync(join(home, '.moltbot'))
            ? join(home, '.moltbot/skills')
            : join(home, '.openclaw/skills');
    case 'cline':
      return join(home, '.agents/skills');
    case 'cursor':
      return join(home, '.cursor/skills');
    case 'gemini-cli':
      return join(home, '.gemini/skills');
    case 'github-copilot':
      return join(home, '.copilot/skills');
    case 'goose':
      return join(configHome, 'goose/skills');
    case 'opencode':
      return join(configHome, 'opencode/skills');
    case 'crush':
      return join(configHome, 'crush/skills');
    case 'antigravity':
      return join(home, '.gemini/antigravity/skills');
    case 'augment':
      return join(home, '.augment/skills');
    case 'codebuddy':
      return join(home, '.codebuddy/skills');
    case 'command-code':
      return join(home, '.commandcode/skills');
    case 'continue':
      return join(home, '.continue/skills');
    case 'cortex':
      return join(home, '.snowflake/cortex/skills');
    case 'droid':
      return join(home, '.factory/skills');
    case 'junie':
      return join(home, '.junie/skills');
    case 'iflow-cli':
      return join(home, '.iflow/skills');
    case 'kilo':
      return join(home, '.kilocode/skills');
    case 'kiro-cli':
      return join(home, '.kiro/skills');
    case 'kode':
      return join(home, '.kode/skills');
    case 'mcpjam':
      return join(home, '.mcpjam/skills');
    case 'mistral-vibe':
      return join(home, '.vibe/skills');
    case 'mux':
      return join(home, '.mux/skills');
    case 'openhands':
      return join(home, '.openhands/skills');
    case 'pi':
      return join(home, '.pi/agent/skills');
    case 'qoder':
      return join(home, '.qoder/skills');
    case 'qwen-code':
      return join(home, '.qwen/skills');
    case 'roo':
      return join(home, '.roo/skills');
    case 'trae':
      return join(home, '.trae/skills');
    case 'trae-cn':
      return join(home, '.trae-cn/skills');
    case 'windsurf':
      return join(home, '.codeium/windsurf/skills');
    case 'zencoder':
      return join(home, '.zencoder/skills');
    case 'neovate':
      return join(home, '.neovate/skills');
    case 'pochi':
      return join(home, '.pochi/skills');
    case 'adal':
      return join(home, '.adal/skills');
  }
}

async function localSkillRoot(path: string): Promise<string> {
  const nested = join(path, '.skill', 'SKILL.md');
  if (existsSync(nested)) return join(path, '.skill');
  const direct = join(path, 'SKILL.md');
  if (existsSync(direct) && path.endsWith('.skill')) return path;
  throw new Error(`InvalidSource: expected .skill/SKILL.md in ${path}`);
}

async function copySourceToDestination(source: SkillSource, destination: string): Promise<void> {
  const parent = join(destination, '..');
  await mkdir(parent, { recursive: true });
  const staging = `${destination}.tmp-${process.pid}`;
  if (existsSync(staging)) await rm(staging, { recursive: true, force: true });
  await mkdir(staging, { recursive: true });

  if (source.type === 'local') {
    const root = await localSkillRoot(source.path);
    await cp(root, staging, { recursive: true });
  } else {
    await writeFile(join(staging, 'SKILL.md'), source.skillMd, 'utf-8');
    for (const file of source.files) {
      if (file.relativePath.includes('..')) throw new Error(`InvalidSource: ${file.relativePath}`);
      const out = join(staging, file.relativePath);
      await mkdir(join(out, '..'), { recursive: true });
      await writeFile(out, file.content);
    }
  }

  if (existsSync(destination)) await rm(destination, { recursive: true, force: true });
  await mkdir(parent, { recursive: true });
  await rm(destination, { recursive: true, force: true });
  await stat(staging);
  await cp(staging, destination, { recursive: true });
  await rm(staging, { recursive: true, force: true });
}

function markerPath(provider: ProviderId, home: string, projectRoot?: string): string | undefined {
  const configHome = process.env.XDG_CONFIG_HOME || join(home, '.config');
  switch (provider) {
    case 'openclaw':
      return [join(home, '.openclaw'), join(home, '.clawdbot'), join(home, '.moltbot')].find((p) => existsSync(p));
    case 'codex':
      return existsSync(process.env.CODEX_HOME || join(home, '.codex'))
        ? process.env.CODEX_HOME || join(home, '.codex')
        : existsSync('/etc/codex')
          ? '/etc/codex'
          : undefined;
    case 'claude-code':
      return process.env.CLAUDE_CONFIG_DIR || join(home, '.claude');
    case 'amp':
      return join(configHome, 'amp');
    case 'goose':
      return join(configHome, 'goose');
    case 'opencode':
      return join(configHome, 'opencode');
    case 'replit':
      return projectRoot ? join(projectRoot, '.replit') : undefined;
    case 'pi':
      return join(home, '.pi/agent');
    case 'cortex':
      return join(home, '.snowflake/cortex');
    case 'windsurf':
      return join(home, '.codeium/windsurf');
    default: {
      const info = providers.find((p) => p.id === provider);
      if (!info) return undefined;
      const first = info.projectPath.replace(/^\./, '').split('/')[0];
      return join(home, `.${first}`);
    }
  }
}

function providerLabel(provider: ProviderId): string {
  return providers.find((p) => p.id === provider)?.displayName ?? provider;
}
