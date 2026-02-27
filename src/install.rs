use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::error::{InstallerError, Result};
use crate::parser::{parse_skill, resolve_local_skill_root};
use crate::providers::{normalize_providers, resolve_provider_dir};
use crate::types::{
    EmbeddedSkill, InstallMethod, InstallRequest, InstallResult, InstallTarget, ProviderId, Scope,
    SkillSource,
};

pub fn resolve_install_target(
    requested_provider: ProviderId,
    scope: Scope,
    project_root: Option<&Path>,
) -> Result<InstallTarget> {
    let target_provider = if crate::providers::is_agents_provider(requested_provider) {
        ProviderId::Universal
    } else {
        requested_provider
    };

    let target_dir = resolve_provider_dir(target_provider, scope, project_root)?;
    Ok(InstallTarget {
        requested_provider,
        target_provider,
        target_dir,
    })
}

pub fn print_install_result(result: &InstallResult) {
    println!("installed skill: {}", result.skill_name);

    for target in &result.installed_targets {
        println!(
            "  {} -> {} ({})",
            target.requested_provider.as_str(),
            target.target_provider.as_str(),
            target.target_dir.display()
        );
    }

    if !result.warnings.is_empty() {
        println!("warnings:");
        for w in &result.warnings {
            println!("  - {w}");
        }
    }
}

pub fn install(request: InstallRequest) -> Result<InstallResult> {
    match request.method {
        InstallMethod::Copy => install_copy(request),
        InstallMethod::Symlink => install_symlink(request),
    }
}

pub fn find_existing_destinations(
    source: &SkillSource,
    providers: &[ProviderId],
    scope: Scope,
    project_root: Option<&Path>,
) -> Result<Vec<PathBuf>> {
    let parsed = parse_skill(source)?;
    let (targets, _) = normalize_providers(providers);

    let mut existing = Vec::new();
    let mut seen = HashSet::new();

    for provider in targets {
        let target = resolve_install_target(provider, scope, project_root)?;
        let destination = target.target_dir.join(&parsed.name);
        if seen.insert(destination.clone()) && destination.exists() {
            existing.push(destination);
        }
    }

    Ok(existing)
}

fn install_copy(request: InstallRequest) -> Result<InstallResult> {
    let parsed = parse_skill(&request.source)?;
    let (providers, normalized_providers) = normalize_providers(&request.providers);

    let mut installed_targets = Vec::new();
    let mut skipped_duplicates = Vec::new();
    let mut warnings = Vec::new();
    let mut seen_paths = HashSet::new();

    for provider in providers {
        let target =
            resolve_install_target(provider, request.scope, request.project_root.as_deref())?;
        let destination = target.target_dir.join(&parsed.name);

        if !seen_paths.insert(destination.clone()) {
            skipped_duplicates.push(destination);
            continue;
        }

        if destination.exists() && !request.force {
            return Err(InstallerError::AlreadyExists { path: destination });
        }

        copy_source_to_destination(&request.source, &destination)?;

        installed_targets.push(InstallTarget {
            requested_provider: provider,
            target_provider: target.target_provider,
            target_dir: destination,
        });
    }

    for (from, to) in &normalized_providers {
        warnings.push(format!(
            "provider '{}' normalized to '{}' shared .agents target",
            from.as_str(),
            to.as_str()
        ));
    }

    Ok(InstallResult {
        skill_name: parsed.name,
        installed_targets,
        normalized_providers,
        skipped_duplicates,
        warnings,
    })
}

fn install_symlink(request: InstallRequest) -> Result<InstallResult> {
    let parsed = parse_skill(&request.source)?;
    let universal_target = resolve_install_target(
        ProviderId::Universal,
        request.scope,
        request.project_root.as_deref(),
    )?;
    let universal_destination = universal_target.target_dir.join(&parsed.name);
    let (providers, normalized_providers) = normalize_providers(&request.providers);

    let mut installed_targets = Vec::new();
    let mut skipped_duplicates = Vec::new();
    let mut warnings = Vec::new();
    let mut seen_paths = HashSet::new();

    if universal_destination.exists() {
        if !request.force {
            return Err(InstallerError::AlreadyExists {
                path: universal_destination.clone(),
            });
        }
        remove_path(&universal_destination)?;
    }

    copy_source_to_destination(&request.source, &universal_destination)?;

    seen_paths.insert(universal_destination.clone());

    for provider in providers {
        let target =
            resolve_install_target(provider, request.scope, request.project_root.as_deref())?;
        let destination = target.target_dir.join(&parsed.name);

        if destination == universal_destination {
            installed_targets.push(InstallTarget {
                requested_provider: provider,
                target_provider: target.target_provider,
                target_dir: destination,
            });
            continue;
        }

        if !seen_paths.insert(destination.clone()) {
            skipped_duplicates.push(destination);
            continue;
        }

        if destination.exists() {
            if !request.force {
                return Err(InstallerError::AlreadyExists { path: destination });
            }
            remove_path(&destination)?;
        }

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|err| InstallerError::IoError {
                path: parent.to_path_buf(),
                message: err.to_string(),
            })?;
        }

        create_dir_symlink(&universal_destination, &destination)?;

        installed_targets.push(InstallTarget {
            requested_provider: provider,
            target_provider: target.target_provider,
            target_dir: destination,
        });
    }

    for (from, to) in &normalized_providers {
        warnings.push(format!(
            "provider '{}' normalized to '{}' shared .agents target",
            from.as_str(),
            to.as_str()
        ));
    }

    Ok(InstallResult {
        skill_name: parsed.name,
        installed_targets,
        normalized_providers,
        skipped_duplicates,
        warnings,
    })
}

fn remove_path(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).map_err(|err| InstallerError::IoError {
        path: path.to_path_buf(),
        message: err.to_string(),
    })?;
    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path).map_err(|err| InstallerError::IoError {
            path: path.to_path_buf(),
            message: err.to_string(),
        })?;
    } else if metadata.is_dir() {
        fs::remove_dir_all(path).map_err(|err| InstallerError::IoError {
            path: path.to_path_buf(),
            message: err.to_string(),
        })?;
    }
    Ok(())
}

#[cfg(unix)]
fn create_dir_symlink(source: &Path, destination: &Path) -> Result<()> {
    std::os::unix::fs::symlink(source, destination).map_err(|err| InstallerError::IoError {
        path: destination.to_path_buf(),
        message: format!(
            "failed to create symlink '{}' -> '{}': {err}",
            destination.display(),
            source.display()
        ),
    })
}

#[cfg(windows)]
fn create_dir_symlink(source: &Path, destination: &Path) -> Result<()> {
    std::os::windows::fs::symlink_dir(source, destination).map_err(|err| InstallerError::IoError {
        path: destination.to_path_buf(),
        message: format!(
            "failed to create symlink '{}' -> '{}': {err}",
            destination.display(),
            source.display()
        ),
    })
}

fn copy_source_to_destination(source: &SkillSource, destination: &Path) -> Result<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| InstallerError::IoError {
            path: destination.to_path_buf(),
            message: "destination has no parent".to_string(),
        })?;

    fs::create_dir_all(parent).map_err(|err| InstallerError::IoError {
        path: parent.to_path_buf(),
        message: err.to_string(),
    })?;

    let staging = parent.join(format!(
        ".{}.tmp-{}",
        destination
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("skill"),
        std::process::id()
    ));

    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|err| InstallerError::IoError {
            path: staging.clone(),
            message: err.to_string(),
        })?;
    }

    fs::create_dir_all(&staging).map_err(|err| InstallerError::IoError {
        path: staging.clone(),
        message: err.to_string(),
    })?;

    match source {
        SkillSource::LocalPath(path) => {
            let root = resolve_local_skill_root(path)?;
            copy_dir_recursive(&root, &staging)?;
        }
        SkillSource::Embedded(embedded) => {
            write_embedded(embedded, &staging)?;
        }
    }

    if destination.exists() {
        fs::remove_dir_all(destination).map_err(|err| InstallerError::IoError {
            path: destination.to_path_buf(),
            message: err.to_string(),
        })?;
    }

    fs::rename(&staging, destination).map_err(|err| InstallerError::IoError {
        path: destination.to_path_buf(),
        message: err.to_string(),
    })?;

    Ok(())
}

fn write_embedded(embedded: &EmbeddedSkill, destination: &Path) -> Result<()> {
    fs::write(destination.join("SKILL.md"), embedded.skill_md.as_bytes()).map_err(|err| {
        InstallerError::IoError {
            path: destination.join("SKILL.md"),
            message: err.to_string(),
        }
    })?;

    for (relative_path, bytes) in &embedded.files {
        if relative_path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(InstallerError::InvalidSource {
                path: relative_path.clone(),
            });
        }

        let file_path = destination.join(relative_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|err| InstallerError::IoError {
                path: parent.to_path_buf(),
                message: err.to_string(),
            })?;
        }
        fs::write(&file_path, bytes).map_err(|err| InstallerError::IoError {
            path: file_path,
            message: err.to_string(),
        })?;
    }

    Ok(())
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<()> {
    for entry in WalkDir::new(source) {
        let entry = entry.map_err(|err| InstallerError::IoError {
            path: source.to_path_buf(),
            message: err.to_string(),
        })?;

        let relative =
            entry
                .path()
                .strip_prefix(source)
                .map_err(|err| InstallerError::IoError {
                    path: entry.path().to_path_buf(),
                    message: err.to_string(),
                })?;

        if relative.as_os_str().is_empty() {
            continue;
        }

        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(|err| InstallerError::IoError {
                path: target,
                message: err.to_string(),
            })?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|err| InstallerError::IoError {
                    path: parent.to_path_buf(),
                    message: err.to_string(),
                })?;
            }
            fs::copy(entry.path(), &target).map_err(|err| InstallerError::IoError {
                path: target,
                message: err.to_string(),
            })?;
        }
    }

    Ok(())
}
