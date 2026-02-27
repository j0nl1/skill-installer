#[cfg(feature = "interactive")]
mod embed;
mod error;
mod install;
#[cfg(feature = "interactive")]
mod interactive;
mod parser;
mod providers;
mod types;

#[cfg(feature = "interactive")]
pub use embed::{load_embedded_skill, rust_embed, Embed};
pub use error::{InstallerError, Result};
pub use install::{
    find_existing_destinations, install, print_install_result, resolve_install_target,
};
#[cfg(feature = "interactive")]
pub use interactive::{
    install_interactive, prompt_provider_selection, prompt_select, InteractiveProviderSelection,
    InteractiveProviderSelectionOptions,
};
pub use parser::parse_skill;
pub use providers::{
    detect_providers, is_agents_provider, normalize_providers, parse_providers_csv,
    supported_providers, ProviderInfo,
};
pub use types::{
    DetectedProvider, EmbeddedSkill, InstallMethod, InstallRequest, InstallResult,
    InstallSkillArgs, InstallTarget, ParsedSkill, ProviderId, Scope, SkillSource,
};
