use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, InstallerError>;

#[derive(Debug, Error)]
pub enum InstallerError {
    #[error("invalid source: expected .skill/SKILL.md in {path}")]
    InvalidSource { path: PathBuf },

    #[error("invalid frontmatter: {message}")]
    InvalidFrontmatter { message: String },

    #[error("missing required frontmatter field: name")]
    MissingName,

    #[error("invalid skill name: {name}")]
    InvalidName { name: String },

    #[error("project scope requires --project-root")]
    ProjectRootRequired,

    #[error("skill already installed at {path}; use --force to overwrite")]
    AlreadyExists { path: PathBuf },

    #[error("unsupported provider: {provider}")]
    UnsupportedProvider { provider: String },

    #[error("installation cancelled by user")]
    PromptCancelled,

    #[error("interactive prompt error: {message}")]
    PromptError { message: String },

    #[error("io error at {path}: {message}")]
    IoError { path: PathBuf, message: String },
}
