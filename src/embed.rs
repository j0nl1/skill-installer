use std::path::PathBuf;

use crate::types::{EmbeddedSkill, SkillSource};

pub use rust_embed;
pub use rust_embed::Embed;

pub fn load_embedded_skill<T: rust_embed::RustEmbed>() -> SkillSource {
    let skill_md_file = T::get("SKILL.md").expect("embedded skill must contain SKILL.md");
    let skill_md = std::str::from_utf8(skill_md_file.data.as_ref())
        .expect("SKILL.md must be valid UTF-8")
        .to_string();

    let files = T::iter()
        .filter(|path| path.as_ref() != "SKILL.md")
        .map(|path| {
            let file = T::get(path.as_ref()).unwrap();
            (PathBuf::from(path.as_ref()), file.data.to_vec())
        })
        .collect();

    SkillSource::Embedded(EmbeddedSkill { skill_md, files })
}
