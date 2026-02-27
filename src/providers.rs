use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::{InstallerError, Result};
use crate::types::{DetectedProvider, ProviderId, Scope};

#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub id: ProviderId,
    pub display_name: &'static str,
    pub uses_agents_dir: bool,
    pub project_path: &'static str,
}

const PROVIDERS: &[ProviderInfo] = &[
    ProviderInfo {
        id: ProviderId::Amp,
        display_name: "Amp",
        uses_agents_dir: true,
        project_path: ".agents/skills",
    },
    ProviderInfo {
        id: ProviderId::Antigravity,
        display_name: "Antigravity",
        uses_agents_dir: false,
        project_path: ".agent/skills",
    },
    ProviderInfo {
        id: ProviderId::Augment,
        display_name: "Augment",
        uses_agents_dir: false,
        project_path: ".augment/skills",
    },
    ProviderInfo {
        id: ProviderId::ClaudeCode,
        display_name: "Claude Code",
        uses_agents_dir: false,
        project_path: ".claude/skills",
    },
    ProviderInfo {
        id: ProviderId::Openclaw,
        display_name: "OpenClaw",
        uses_agents_dir: false,
        project_path: "skills",
    },
    ProviderInfo {
        id: ProviderId::Cline,
        display_name: "Cline",
        uses_agents_dir: true,
        project_path: ".agents/skills",
    },
    ProviderInfo {
        id: ProviderId::Codebuddy,
        display_name: "CodeBuddy",
        uses_agents_dir: false,
        project_path: ".codebuddy/skills",
    },
    ProviderInfo {
        id: ProviderId::Codex,
        display_name: "Codex",
        uses_agents_dir: true,
        project_path: ".agents/skills",
    },
    ProviderInfo {
        id: ProviderId::CommandCode,
        display_name: "Command Code",
        uses_agents_dir: false,
        project_path: ".commandcode/skills",
    },
    ProviderInfo {
        id: ProviderId::Continue,
        display_name: "Continue",
        uses_agents_dir: false,
        project_path: ".continue/skills",
    },
    ProviderInfo {
        id: ProviderId::Cortex,
        display_name: "Cortex Code",
        uses_agents_dir: false,
        project_path: ".cortex/skills",
    },
    ProviderInfo {
        id: ProviderId::Crush,
        display_name: "Crush",
        uses_agents_dir: false,
        project_path: ".crush/skills",
    },
    ProviderInfo {
        id: ProviderId::Cursor,
        display_name: "Cursor",
        uses_agents_dir: true,
        project_path: ".agents/skills",
    },
    ProviderInfo {
        id: ProviderId::Droid,
        display_name: "Droid",
        uses_agents_dir: false,
        project_path: ".factory/skills",
    },
    ProviderInfo {
        id: ProviderId::GeminiCli,
        display_name: "Gemini CLI",
        uses_agents_dir: true,
        project_path: ".agents/skills",
    },
    ProviderInfo {
        id: ProviderId::GithubCopilot,
        display_name: "GitHub Copilot",
        uses_agents_dir: true,
        project_path: ".agents/skills",
    },
    ProviderInfo {
        id: ProviderId::Goose,
        display_name: "Goose",
        uses_agents_dir: false,
        project_path: ".goose/skills",
    },
    ProviderInfo {
        id: ProviderId::Junie,
        display_name: "Junie",
        uses_agents_dir: false,
        project_path: ".junie/skills",
    },
    ProviderInfo {
        id: ProviderId::IflowCli,
        display_name: "iFlow CLI",
        uses_agents_dir: false,
        project_path: ".iflow/skills",
    },
    ProviderInfo {
        id: ProviderId::Kilo,
        display_name: "Kilo Code",
        uses_agents_dir: false,
        project_path: ".kilocode/skills",
    },
    ProviderInfo {
        id: ProviderId::KimiCli,
        display_name: "Kimi Code CLI",
        uses_agents_dir: true,
        project_path: ".agents/skills",
    },
    ProviderInfo {
        id: ProviderId::KiroCli,
        display_name: "Kiro CLI",
        uses_agents_dir: false,
        project_path: ".kiro/skills",
    },
    ProviderInfo {
        id: ProviderId::Kode,
        display_name: "Kode",
        uses_agents_dir: false,
        project_path: ".kode/skills",
    },
    ProviderInfo {
        id: ProviderId::Mcpjam,
        display_name: "MCPJam",
        uses_agents_dir: false,
        project_path: ".mcpjam/skills",
    },
    ProviderInfo {
        id: ProviderId::MistralVibe,
        display_name: "Mistral Vibe",
        uses_agents_dir: false,
        project_path: ".vibe/skills",
    },
    ProviderInfo {
        id: ProviderId::Mux,
        display_name: "Mux",
        uses_agents_dir: false,
        project_path: ".mux/skills",
    },
    ProviderInfo {
        id: ProviderId::Opencode,
        display_name: "OpenCode",
        uses_agents_dir: true,
        project_path: ".agents/skills",
    },
    ProviderInfo {
        id: ProviderId::Openhands,
        display_name: "OpenHands",
        uses_agents_dir: false,
        project_path: ".openhands/skills",
    },
    ProviderInfo {
        id: ProviderId::Pi,
        display_name: "Pi",
        uses_agents_dir: false,
        project_path: ".pi/skills",
    },
    ProviderInfo {
        id: ProviderId::Qoder,
        display_name: "Qoder",
        uses_agents_dir: false,
        project_path: ".qoder/skills",
    },
    ProviderInfo {
        id: ProviderId::QwenCode,
        display_name: "Qwen Code",
        uses_agents_dir: false,
        project_path: ".qwen/skills",
    },
    ProviderInfo {
        id: ProviderId::Replit,
        display_name: "Replit",
        uses_agents_dir: true,
        project_path: ".agents/skills",
    },
    ProviderInfo {
        id: ProviderId::Roo,
        display_name: "Roo Code",
        uses_agents_dir: false,
        project_path: ".roo/skills",
    },
    ProviderInfo {
        id: ProviderId::Trae,
        display_name: "Trae",
        uses_agents_dir: false,
        project_path: ".trae/skills",
    },
    ProviderInfo {
        id: ProviderId::TraeCn,
        display_name: "Trae CN",
        uses_agents_dir: false,
        project_path: ".trae/skills",
    },
    ProviderInfo {
        id: ProviderId::Windsurf,
        display_name: "Windsurf",
        uses_agents_dir: false,
        project_path: ".windsurf/skills",
    },
    ProviderInfo {
        id: ProviderId::Zencoder,
        display_name: "Zencoder",
        uses_agents_dir: false,
        project_path: ".zencoder/skills",
    },
    ProviderInfo {
        id: ProviderId::Neovate,
        display_name: "Neovate",
        uses_agents_dir: false,
        project_path: ".neovate/skills",
    },
    ProviderInfo {
        id: ProviderId::Pochi,
        display_name: "Pochi",
        uses_agents_dir: false,
        project_path: ".pochi/skills",
    },
    ProviderInfo {
        id: ProviderId::Adal,
        display_name: "AdaL",
        uses_agents_dir: false,
        project_path: ".adal/skills",
    },
    ProviderInfo {
        id: ProviderId::Universal,
        display_name: "Universal",
        uses_agents_dir: true,
        project_path: ".agents/skills",
    },
];

pub fn supported_providers() -> &'static [ProviderInfo] {
    PROVIDERS
}

pub fn is_agents_provider(provider: ProviderId) -> bool {
    provider_info(provider)
        .map(|p| p.uses_agents_dir)
        .unwrap_or(false)
}

pub fn normalize_providers(
    providers: &[ProviderId],
) -> (Vec<ProviderId>, Vec<(ProviderId, ProviderId)>) {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();

    for &provider in providers {
        let target = if is_agents_provider(provider) {
            ProviderId::Universal
        } else {
            provider
        };
        if target != provider {
            normalized.push((provider, target));
        }
        if seen.insert(target) {
            out.push(target);
        }
    }

    (out, normalized)
}

pub fn parse_providers_csv(raw: &str) -> Result<Vec<ProviderId>> {
    if raw.trim() == "*" {
        return Ok(supported_providers().iter().map(|p| p.id).collect());
    }

    let mut out = Vec::new();
    for token in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let provider =
            ProviderId::from_str(token).ok_or_else(|| InstallerError::UnsupportedProvider {
                provider: token.to_string(),
            })?;
        out.push(provider);
    }

    if out.is_empty() {
        return Err(InstallerError::UnsupportedProvider {
            provider: "(empty)".to_string(),
        });
    }

    Ok(out)
}

pub fn detect_providers(project_root: Option<&Path>) -> Vec<DetectedProvider> {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("~"));
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".config"));

    let mut detected = Vec::new();
    for provider in supported_providers() {
        if provider.id == ProviderId::Universal {
            continue;
        }

        if let Some(reason) = detect_provider(provider.id, &home, &config_home, project_root) {
            detected.push(DetectedProvider {
                provider: provider.id,
                reason,
            });
        }
    }

    detected
}

fn detect_provider(
    provider: ProviderId,
    home: &Path,
    config_home: &Path,
    project_root: Option<&Path>,
) -> Option<String> {
    let marker = match provider {
        ProviderId::Openclaw => {
            let candidates = [
                home.join(".openclaw"),
                home.join(".clawdbot"),
                home.join(".moltbot"),
            ];
            return candidates
                .into_iter()
                .find(|p| p.exists())
                .map(|p| format!("found {}", p.display()));
        }
        ProviderId::Codex => {
            let codex_home = std::env::var("CODEX_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| home.join(".codex"));
            if codex_home.exists() {
                return Some(format!("found {}", codex_home.display()));
            }
            if Path::new("/etc/codex").exists() {
                return Some("found /etc/codex".to_string());
            }
            codex_home
        }
        ProviderId::ClaudeCode => std::env::var("CLAUDE_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".claude")),
        ProviderId::Amp => config_home.join("amp"),
        ProviderId::Goose => config_home.join("goose"),
        ProviderId::Opencode => config_home.join("opencode"),
        ProviderId::KimiCli => home.join(".kimi"),
        ProviderId::Replit => {
            if let Some(root) = project_root {
                let p = root.join(".replit");
                if p.exists() {
                    return Some(format!("found {}", p.display()));
                }
            }
            return None;
        }
        ProviderId::Pi => home.join(".pi/agent"),
        ProviderId::Cortex => home.join(".snowflake/cortex"),
        ProviderId::Windsurf => home.join(".codeium/windsurf"),
        _ => {
            let base = project_path_for(provider)
                .trim_start_matches('.')
                .trim_start_matches('/');
            let first = base.split('/').next().unwrap_or(base);
            home.join(format!(".{}", first))
        }
    };

    if marker.exists() {
        return Some(format!("found {}", marker.display()));
    }

    if let Some(root) = project_root {
        let p = root.join(project_path_for(provider));
        if p.exists() {
            return Some(format!("found {}", p.display()));
        }
    }

    None
}

pub fn resolve_provider_dir(
    provider: ProviderId,
    scope: Scope,
    project_root: Option<&Path>,
) -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("~"));
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".config"));

    match scope {
        Scope::Project => {
            let root = project_root.ok_or(InstallerError::ProjectRootRequired)?;
            Ok(root.join(project_path_for(provider)))
        }
        Scope::User => Ok(user_path_for(provider, &home, &config_home)),
    }
}

pub fn project_path_for(provider: ProviderId) -> &'static str {
    provider_info(provider)
        .map(|p| p.project_path)
        .unwrap_or(".agents/skills")
}

fn provider_info(provider: ProviderId) -> Option<&'static ProviderInfo> {
    supported_providers().iter().find(|p| p.id == provider)
}

fn user_path_for(provider: ProviderId, home: &Path, config_home: &Path) -> PathBuf {
    match provider {
        ProviderId::Universal | ProviderId::Amp | ProviderId::KimiCli | ProviderId::Replit => {
            config_home.join("agents/skills")
        }
        ProviderId::Antigravity => home.join(".gemini/antigravity/skills"),
        ProviderId::Augment => home.join(".augment/skills"),
        ProviderId::ClaudeCode => {
            let claude_home = std::env::var("CLAUDE_CONFIG_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| home.join(".claude"));
            claude_home.join("skills")
        }
        ProviderId::Openclaw => {
            if home.join(".openclaw").exists() {
                home.join(".openclaw/skills")
            } else if home.join(".clawdbot").exists() {
                home.join(".clawdbot/skills")
            } else if home.join(".moltbot").exists() {
                home.join(".moltbot/skills")
            } else {
                home.join(".openclaw/skills")
            }
        }
        ProviderId::Cline => home.join(".agents/skills"),
        ProviderId::Codebuddy => home.join(".codebuddy/skills"),
        ProviderId::Codex => {
            let codex_home = std::env::var("CODEX_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| home.join(".codex"));
            codex_home.join("skills")
        }
        ProviderId::CommandCode => home.join(".commandcode/skills"),
        ProviderId::Continue => home.join(".continue/skills"),
        ProviderId::Cortex => home.join(".snowflake/cortex/skills"),
        ProviderId::Crush => config_home.join("crush/skills"),
        ProviderId::Cursor => home.join(".cursor/skills"),
        ProviderId::Droid => home.join(".factory/skills"),
        ProviderId::GeminiCli => home.join(".gemini/skills"),
        ProviderId::GithubCopilot => home.join(".copilot/skills"),
        ProviderId::Goose => config_home.join("goose/skills"),
        ProviderId::Junie => home.join(".junie/skills"),
        ProviderId::IflowCli => home.join(".iflow/skills"),
        ProviderId::Kilo => home.join(".kilocode/skills"),
        ProviderId::KiroCli => home.join(".kiro/skills"),
        ProviderId::Kode => home.join(".kode/skills"),
        ProviderId::Mcpjam => home.join(".mcpjam/skills"),
        ProviderId::MistralVibe => home.join(".vibe/skills"),
        ProviderId::Mux => home.join(".mux/skills"),
        ProviderId::Opencode => config_home.join("opencode/skills"),
        ProviderId::Openhands => home.join(".openhands/skills"),
        ProviderId::Pi => home.join(".pi/agent/skills"),
        ProviderId::Qoder => home.join(".qoder/skills"),
        ProviderId::QwenCode => home.join(".qwen/skills"),
        ProviderId::Roo => home.join(".roo/skills"),
        ProviderId::Trae => home.join(".trae/skills"),
        ProviderId::TraeCn => home.join(".trae-cn/skills"),
        ProviderId::Windsurf => home.join(".codeium/windsurf/skills"),
        ProviderId::Zencoder => home.join(".zencoder/skills"),
        ProviderId::Neovate => home.join(".neovate/skills"),
        ProviderId::Pochi => home.join(".pochi/skills"),
        ProviderId::Adal => home.join(".adal/skills"),
    }
}
