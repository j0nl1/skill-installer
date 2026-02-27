use std::fs;

use skillinstaller::{
    detect_providers, install, parse_skill, InstallMethod, InstallRequest, InstallerError,
    ProviderId, Scope, SkillSource,
};
use tempfile::TempDir;

fn make_skill_fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let skill_root = dir.path().join(".skill");
    fs::create_dir_all(skill_root.join("scripts")).unwrap();
    fs::write(
        skill_root.join("SKILL.md"),
        "---\nname: demo-skill\ndescription: Demo\nmetadata:\n  author: acme\n---\nUse this skill.",
    )
    .unwrap();
    fs::write(skill_root.join("scripts/run.sh"), "echo hi").unwrap();
    dir
}

#[test]
fn parse_skill_from_local_path() {
    let fixture = make_skill_fixture();
    let parsed = parse_skill(&SkillSource::LocalPath(fixture.path().to_path_buf())).unwrap();

    assert_eq!(parsed.name, "demo-skill");
    assert_eq!(parsed.description.as_deref(), Some("Demo"));
    assert_eq!(
        parsed
            .metadata
            .as_ref()
            .and_then(|m| m.get("author"))
            .map(String::as_str),
        Some("acme")
    );
}

#[test]
fn install_copies_full_skill_payload_and_normalizes_agents_providers() {
    let fixture = make_skill_fixture();
    let project = TempDir::new().unwrap();

    let result = install(InstallRequest {
        source: SkillSource::LocalPath(fixture.path().to_path_buf()),
        providers: vec![ProviderId::Cursor, ProviderId::ClaudeCode],
        scope: Scope::Project,
        project_root: Some(project.path().to_path_buf()),
        method: InstallMethod::Copy,
        force: false,
    })
    .unwrap();

    assert_eq!(result.skill_name, "demo-skill");
    assert!(result
        .normalized_providers
        .iter()
        .any(|(from, to)| *from == ProviderId::Cursor && *to == ProviderId::Universal));

    let universal_skill = project.path().join(".agents/skills/demo-skill");
    let claude_skill = project.path().join(".claude/skills/demo-skill");

    assert!(universal_skill.join("SKILL.md").exists());
    assert!(universal_skill.join("scripts/run.sh").exists());
    assert!(claude_skill.join("SKILL.md").exists());
}

#[test]
fn install_fails_without_force_if_destination_exists() {
    let fixture = make_skill_fixture();
    let project = TempDir::new().unwrap();

    let request = InstallRequest {
        source: SkillSource::LocalPath(fixture.path().to_path_buf()),
        providers: vec![ProviderId::ClaudeCode],
        scope: Scope::Project,
        project_root: Some(project.path().to_path_buf()),
        method: InstallMethod::Copy,
        force: false,
    };

    install(request.clone()).unwrap();
    let second = install(request);

    match second {
        Err(InstallerError::AlreadyExists { .. }) => {}
        other => panic!("expected AlreadyExists, got {other:?}"),
    }
}

#[test]
fn install_symlink_copies_to_universal_and_links_other_providers() {
    let fixture = make_skill_fixture();
    let project = TempDir::new().unwrap();

    let result = install(InstallRequest {
        source: SkillSource::LocalPath(fixture.path().to_path_buf()),
        providers: vec![ProviderId::ClaudeCode],
        scope: Scope::Project,
        project_root: Some(project.path().to_path_buf()),
        method: InstallMethod::Symlink,
        force: false,
    })
    .unwrap();

    assert_eq!(result.skill_name, "demo-skill");

    let universal_skill = project.path().join(".agents/skills/demo-skill");
    let claude_skill = project.path().join(".claude/skills/demo-skill");

    let universal_meta = fs::symlink_metadata(&universal_skill).unwrap();
    assert!(universal_meta.is_dir());
    assert!(!universal_meta.file_type().is_symlink());

    let claude_meta = fs::symlink_metadata(&claude_skill).unwrap();
    assert!(claude_meta.file_type().is_symlink());
    assert_eq!(fs::read_link(&claude_skill).unwrap(), universal_skill);
    assert!(claude_skill.join("SKILL.md").exists());
    assert!(claude_skill.join("scripts/run.sh").exists());
}

#[test]
fn detect_providers_returns_empty_in_clean_temp_home() {
    let temp_home = TempDir::new().unwrap();
    std::env::set_var("HOME", temp_home.path());
    std::env::remove_var("XDG_CONFIG_HOME");

    let detected = detect_providers(Some(temp_home.path()));
    assert!(detected.is_empty());
}
