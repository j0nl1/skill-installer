use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::Value;

use crate::error::{InstallerError, Result};
use crate::types::{ParsedSkill, SkillSource};

pub fn parse_skill(source: &SkillSource) -> Result<ParsedSkill> {
    let skill_md = match source {
        SkillSource::LocalPath(path) => {
            let root = resolve_local_skill_root(path)?;
            fs::read_to_string(root.join("SKILL.md")).map_err(|err| InstallerError::IoError {
                path: root.join("SKILL.md"),
                message: err.to_string(),
            })?
        }
        SkillSource::Embedded(embedded) => embedded.skill_md.clone(),
    };

    let (frontmatter, body) = split_frontmatter(&skill_md)?;
    let yaml: Value =
        serde_yaml::from_str(frontmatter).map_err(|err| InstallerError::InvalidFrontmatter {
            message: err.to_string(),
        })?;

    let map = yaml
        .as_mapping()
        .ok_or_else(|| InstallerError::InvalidFrontmatter {
            message: "frontmatter must be a YAML mapping".to_string(),
        })?;

    let name = map
        .get(Value::from("name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(InstallerError::MissingName)?
        .to_string();

    validate_skill_name(&name)?;

    let description = map
        .get(Value::from("description"))
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let allowed_tools = map
        .get(Value::from("allowed-tools"))
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let metadata = map
        .get(Value::from("metadata"))
        .and_then(Value::as_mapping)
        .map(|meta| {
            let mut out = BTreeMap::new();
            for (k, v) in meta {
                if let (Some(key), Some(value)) = (k.as_str(), v.as_str()) {
                    out.insert(key.to_string(), value.to_string());
                }
            }
            out
        })
        .filter(|m| !m.is_empty());

    Ok(ParsedSkill {
        name,
        description,
        metadata,
        allowed_tools,
        body: body.to_string(),
    })
}

pub(crate) fn resolve_local_skill_root(path: &Path) -> Result<PathBuf> {
    let direct = path.join("SKILL.md");
    if path.ends_with(".skill") && direct.exists() {
        return Ok(path.to_path_buf());
    }

    let nested = path.join(".skill");
    if nested.join("SKILL.md").exists() {
        return Ok(nested);
    }

    Err(InstallerError::InvalidSource {
        path: path.to_path_buf(),
    })
}

fn split_frontmatter(content: &str) -> Result<(&str, &str)> {
    if !content.starts_with("---\n") {
        return Err(InstallerError::InvalidFrontmatter {
            message: "missing opening frontmatter delimiter".to_string(),
        });
    }

    let after = &content[4..];
    let end = after
        .find("\n---\n")
        .ok_or_else(|| InstallerError::InvalidFrontmatter {
            message: "missing closing frontmatter delimiter".to_string(),
        })?;

    let frontmatter = &after[..end];
    let body = &after[(end + 5)..];
    Ok((frontmatter, body))
}

fn validate_skill_name(name: &str) -> Result<()> {
    let invalid = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    if name.chars().any(|c| invalid.contains(&c)) || name == "." || name == ".." {
        return Err(InstallerError::InvalidName {
            name: name.to_string(),
        });
    }
    Ok(())
}
