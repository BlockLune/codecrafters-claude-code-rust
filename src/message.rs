pub fn build_system_prompt(appending: Option<&str>) -> String {
    let mut system_prompt = String::from(include_str!("./config/prompt/system_prompt.md").trim());
    if let Some(appending) = appending {
        system_prompt.push_str(&format!("\n\n{}\n", appending.trim()));
    }
    system_prompt
}
