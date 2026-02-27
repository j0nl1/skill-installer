use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ProviderId {
    Amp,
    Antigravity,
    Augment,
    ClaudeCode,
    Openclaw,
    Cline,
    Codebuddy,
    Codex,
    CommandCode,
    Continue,
    Cortex,
    Crush,
    Cursor,
    Droid,
    GeminiCli,
    GithubCopilot,
    Goose,
    Junie,
    IflowCli,
    Kilo,
    KimiCli,
    KiroCli,
    Kode,
    Mcpjam,
    MistralVibe,
    Mux,
    Opencode,
    Openhands,
    Pi,
    Qoder,
    QwenCode,
    Replit,
    Roo,
    Trae,
    TraeCn,
    Windsurf,
    Zencoder,
    Neovate,
    Pochi,
    Adal,
    Universal,
}

impl ProviderId {
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderId::Amp => "amp",
            ProviderId::Antigravity => "antigravity",
            ProviderId::Augment => "augment",
            ProviderId::ClaudeCode => "claude-code",
            ProviderId::Openclaw => "openclaw",
            ProviderId::Cline => "cline",
            ProviderId::Codebuddy => "codebuddy",
            ProviderId::Codex => "codex",
            ProviderId::CommandCode => "command-code",
            ProviderId::Continue => "continue",
            ProviderId::Cortex => "cortex",
            ProviderId::Crush => "crush",
            ProviderId::Cursor => "cursor",
            ProviderId::Droid => "droid",
            ProviderId::GeminiCli => "gemini-cli",
            ProviderId::GithubCopilot => "github-copilot",
            ProviderId::Goose => "goose",
            ProviderId::Junie => "junie",
            ProviderId::IflowCli => "iflow-cli",
            ProviderId::Kilo => "kilo",
            ProviderId::KimiCli => "kimi-cli",
            ProviderId::KiroCli => "kiro-cli",
            ProviderId::Kode => "kode",
            ProviderId::Mcpjam => "mcpjam",
            ProviderId::MistralVibe => "mistral-vibe",
            ProviderId::Mux => "mux",
            ProviderId::Opencode => "opencode",
            ProviderId::Openhands => "openhands",
            ProviderId::Pi => "pi",
            ProviderId::Qoder => "qoder",
            ProviderId::QwenCode => "qwen-code",
            ProviderId::Replit => "replit",
            ProviderId::Roo => "roo",
            ProviderId::Trae => "trae",
            ProviderId::TraeCn => "trae-cn",
            ProviderId::Windsurf => "windsurf",
            ProviderId::Zencoder => "zencoder",
            ProviderId::Neovate => "neovate",
            ProviderId::Pochi => "pochi",
            ProviderId::Adal => "adal",
            ProviderId::Universal => "universal",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        Some(match value {
            "amp" => ProviderId::Amp,
            "antigravity" => ProviderId::Antigravity,
            "augment" => ProviderId::Augment,
            "claude-code" => ProviderId::ClaudeCode,
            "openclaw" => ProviderId::Openclaw,
            "cline" => ProviderId::Cline,
            "codebuddy" => ProviderId::Codebuddy,
            "codex" => ProviderId::Codex,
            "command-code" => ProviderId::CommandCode,
            "continue" => ProviderId::Continue,
            "cortex" => ProviderId::Cortex,
            "crush" => ProviderId::Crush,
            "cursor" => ProviderId::Cursor,
            "droid" => ProviderId::Droid,
            "gemini-cli" => ProviderId::GeminiCli,
            "github-copilot" => ProviderId::GithubCopilot,
            "goose" => ProviderId::Goose,
            "junie" => ProviderId::Junie,
            "iflow-cli" => ProviderId::IflowCli,
            "kilo" => ProviderId::Kilo,
            "kimi-cli" => ProviderId::KimiCli,
            "kiro-cli" => ProviderId::KiroCli,
            "kode" => ProviderId::Kode,
            "mcpjam" => ProviderId::Mcpjam,
            "mistral-vibe" => ProviderId::MistralVibe,
            "mux" => ProviderId::Mux,
            "opencode" => ProviderId::Opencode,
            "openhands" => ProviderId::Openhands,
            "pi" => ProviderId::Pi,
            "qoder" => ProviderId::Qoder,
            "qwen-code" => ProviderId::QwenCode,
            "replit" => ProviderId::Replit,
            "roo" => ProviderId::Roo,
            "trae" => ProviderId::Trae,
            "trae-cn" => ProviderId::TraeCn,
            "windsurf" => ProviderId::Windsurf,
            "zencoder" => ProviderId::Zencoder,
            "neovate" => ProviderId::Neovate,
            "pochi" => ProviderId::Pochi,
            "adal" => ProviderId::Adal,
            "universal" => ProviderId::Universal,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Scope {
    User,
    Project,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum InstallMethod {
    Symlink,
    Copy,
}

#[derive(Debug, Clone)]
pub struct EmbeddedSkill {
    pub skill_md: String,
    pub files: Vec<(PathBuf, Vec<u8>)>,
}

#[derive(Debug, Clone)]
pub enum SkillSource {
    LocalPath(PathBuf),
    Embedded(EmbeddedSkill),
}

#[derive(Debug, Clone)]
pub struct ParsedSkill {
    pub name: String,
    pub description: Option<String>,
    pub metadata: Option<BTreeMap<String, String>>,
    pub allowed_tools: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct InstallRequest {
    pub source: SkillSource,
    pub providers: Vec<ProviderId>,
    pub scope: Scope,
    pub project_root: Option<PathBuf>,
    pub method: InstallMethod,
    pub force: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallTarget {
    pub requested_provider: ProviderId,
    pub target_provider: ProviderId,
    pub target_dir: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallResult {
    pub skill_name: String,
    pub installed_targets: Vec<InstallTarget>,
    pub normalized_providers: Vec<(ProviderId, ProviderId)>,
    pub skipped_duplicates: Vec<PathBuf>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DetectedProvider {
    pub provider: ProviderId,
    pub reason: String,
}

#[derive(Debug, Clone, clap::Args)]
pub struct InstallSkillArgs {
    /// Providers to target (comma-separated). Use '*' for all.
    #[arg(long)]
    pub providers: Option<String>,

    /// Install scope
    #[arg(long, value_enum)]
    pub scope: Option<Scope>,

    /// Project root; defaults to current directory when scope is project
    #[arg(long)]
    pub project_root: Option<PathBuf>,

    /// Installation method
    #[arg(long, value_enum)]
    pub method: Option<InstallMethod>,

    /// Overwrite existing destination skill folders
    #[arg(long, default_value_t = false)]
    pub force: bool,
}
