use crate::skill::LoadedSkills;
use anyhow::Result;
use std::path::PathBuf;

pub fn build_system_prompt(appending: Option<&str>) -> Result<String> {
    let skills_path = PathBuf::from("./.claude/skills");
    let skills = LoadedSkills::load_from(&skills_path)?;

    let mut system_prompt = String::from(include_str!("./config/prompt/system_prompt.md").trim());
    if let Some(appending) = appending {
        system_prompt.push_str(&format!("\n\n{}\n", appending.trim()));
    }
    if !skills.is_empty() {
        system_prompt.push_str(&format!("\n\n{}\n", skills.xml()?));
    }

    Ok(system_prompt)
}
