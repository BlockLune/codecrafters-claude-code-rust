use std::path::PathBuf;
use std::env;
use crate::skill::LoadedSkills;

pub fn build_system_prompt(appending: Option<&str>) -> String {
    let home_path_str = env::var("HOME").unwrap();
    let skills_path = PathBuf::from(home_path_str).join(".agents/skills");
    let skills = LoadedSkills::load_from(&skills_path);

    let mut system_prompt = String::from(include_str!("./config/prompt/system_prompt.md").trim());
    if let Some(appending) = appending {
        system_prompt.push_str(&format!("\n\n{}\n", appending.trim()));
    }
    if !skills.is_empty() {
        system_prompt.push_str(&format!("\n\n{}\n", skills.xml()));
    }

    system_prompt
}
