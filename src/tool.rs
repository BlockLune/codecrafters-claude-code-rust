use std::fs;
use std::process::Command;
use thiserror::Error;

pub fn get_tools() -> Vec<serde_json::Value> {
    vec![
        serde_json::from_str(include_str!("./config/tool/read.json")).unwrap(),
        serde_json::from_str(include_str!("./config/tool/write.json")).unwrap(),
        serde_json::from_str(include_str!("./config/tool/bash.json")).unwrap(),
    ]
}

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Unknown tool: {0}")]
    UnknownTool(String),
    #[error("Failed to parse arguments")]
    FailedToParseArguments,
    #[error("Missing argument: {0}")]
    MissingArg(&'static str),
    #[error("Failed to read: {0}")]
    FailedToRead(String),
    #[error("Failed to write: {0}")]
    FailedToWrite(String),
    #[error("Failed to run bash")]
    FailedToRunBash,
}

fn read_tool(file_path: &str) -> Result<String, ToolError> {
    fs::read_to_string(file_path).map_err(|_| ToolError::FailedToRead(file_path.to_owned()))
}

fn write_tool(file_path: &str, content: &str) -> Result<String, ToolError> {
    fs::write(file_path, content)
        .map(|_| format!("Successfully wrote to file {}", file_path))
        .map_err(|_| ToolError::FailedToWrite(file_path.to_owned()))
}

fn bash_tool(command: &str) -> Result<String, ToolError> {
    Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()
        .map(|output| {
            [
                format!("Exit code: {:?}", output.status.code()),
                format!("stdout: {}", String::from_utf8_lossy(&output.stdout)),
                format!("stderr: {}", String::from_utf8_lossy(&output.stderr)),
            ]
            .join("\n")
        })
        .map_err(|_| ToolError::FailedToRunBash)
}

pub fn execute(id: &str, name: &str, args: &str) -> Result<serde_json::Value, ToolError> {
    let build_tool_msg = |content: &str| -> serde_json::Value {
        serde_json::json!({
            "role": "tool",
            "tool_call_id": id,
            "content": content,
        })
    };

    let args: serde_json::Value =
        serde_json::from_str(args).map_err(|_| ToolError::FailedToParseArguments)?;

    match name {
        "Read" => {
            let file_path = args["file_path"]
                .as_str()
                .ok_or(ToolError::MissingArg("file_path"))?;
            Ok(build_tool_msg(&read_tool(file_path)?))
        }
        "Write" => {
            let file_path = args["file_path"]
                .as_str()
                .ok_or(ToolError::MissingArg("file_path"))?;
            let content = args["content"]
                .as_str()
                .ok_or(ToolError::MissingArg("content"))?;
            Ok(build_tool_msg(&write_tool(file_path, content)?))
        }
        "Bash" => {
            let command = args["command"]
                .as_str()
                .ok_or(ToolError::MissingArg("command"))?;
            Ok(build_tool_msg(&bash_tool(command)?))
        }
        _ => Err(ToolError::UnknownTool(name.to_owned())),
    }
}
