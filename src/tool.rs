use std::fs;

pub fn get_tools() -> Vec<serde_json::Value> {
    vec![
        serde_json::from_str(include_str!("./tools/read.json")).unwrap(),
        serde_json::from_str(include_str!("./tools/write.json")).unwrap(),
    ]
}

pub fn read_tool(file_path: &str) -> String {
    fs::read_to_string(file_path).unwrap_or_default()
}

pub fn write_tool(file_path: &str, content: &str) {
    fs::write(file_path, content).unwrap()
}
