use std::fs;
use std::process::Command;

pub fn get_tools() -> Vec<serde_json::Value> {
    vec![
        serde_json::from_str(include_str!("./config/tool/read.json")).unwrap(),
        serde_json::from_str(include_str!("./config/tool/write.json")).unwrap(),
        serde_json::from_str(include_str!("./config/tool/bash.json")).unwrap(),
    ]
}

fn read_tool(file_path: &str) -> String {
    fs::read_to_string(file_path).unwrap_or_default()
}

fn write_tool(file_path: &str, content: &str) -> String {
    match fs::write(file_path, content) {
        Ok(_) => format!("Successfully wrote to file {}", file_path),
        Err(_) => format!("Failed to write to file {}", file_path),
    }
}

fn bash_tool(command: &str) -> String {
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

pub fn execute_tool_and_get_msg(
    id: &str,
    name: &str,
    args: &serde_json::Value,
) -> Option<serde_json::Value> {
    let build_tool_msg = |content: &str| -> serde_json::Value {
        serde_json::json!({
            "role": "tool",
            "tool_call_id": id,
            "content": content,
        })
    };

    match name {
        "Read" => {
            let file_path = args["file_path"].as_str()?;
            Some(build_tool_msg(&read_tool(file_path)))
        }
        "Write" => {
            let file_path = args["file_path"].as_str()?;
            let content = args["content"].as_str()?;
            Some(build_tool_msg(&write_tool(file_path, content)))
        }
        "Bash" => {
            let command = args["command"].as_str()?;
            Some(build_tool_msg(&bash_tool(command)))
        }
        _ => None,
    }
}
