use std::fs;
use std::process::Command;

pub fn get_tools() -> Vec<serde_json::Value> {
    vec![
        serde_json::from_str(include_str!("./config/tool/read.json")).unwrap(),
        serde_json::from_str(include_str!("./config/tool/write.json")).unwrap(),
        serde_json::from_str(include_str!("./config/tool/bash.json")).unwrap(),
    ]
}

pub fn read_tool(file_path: &str) -> String {
    fs::read_to_string(file_path).unwrap_or_default()
}

pub fn write_tool(file_path: &str, content: &str) -> String {
    match fs::write(file_path, content) {
        Ok(_) => format!("Successfully wrote to file {}", file_path),
        Err(_) => format!("Failed to write to file {}", file_path),
    }
}

pub fn bash_tool(command: &str) -> String {
    match Command::new("bash").arg("-c").arg(command).output() {
        Ok(output) => [
            format!("Exit code: {:?}", output.status.code()),
            format!("stdout: {}", String::from_utf8_lossy(&output.stdout)),
            format!("stderr: {}", String::from_utf8_lossy(&output.stderr)),
        ]
        .join("\n"),
        Err(_) => "Failed to execute process".to_string(),
    }
}
