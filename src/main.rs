use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::{env, process};

mod tool;
use tool::{get_tools, read_tool, write_tool, bash_tool};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
    #[arg(long)]
    model: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);
    let tools = get_tools();

    let first_msg = json!({ "role": "user", "content": args.prompt });
    let mut msgs = vec![first_msg];

    let model = args.model.unwrap_or("anthropic/claude-haiku-4.5".to_string());

    loop {
        let response: Value = client
            .chat()
            .create_byot(json!({
                "messages": msgs,
                "model": model,
                "tools": tools
            }))
            .await?;

        let msg = (&response["choices"][0]["message"]).clone();
        msgs.push(msg.clone());

        if let Some(tool_calls) = msg["tool_calls"].as_array() {
            for tool_call in tool_calls {
                let Some(function_name) = tool_call["function"]["name"].as_str() else {
                    continue;
                };
                let Some(function_arguments) = tool_call["function"]["arguments"].as_str() else {
                    continue;
                };

                match function_name {
                    "Read" => {
                        let args: Value =
                            serde_json::from_str(function_arguments).unwrap_or_default();
                        if let Some(file_path) = args["file_path"].as_str() {
                            let tool_msg = json!({
                                "role": "tool",
                                "tool_call_id": tool_call["id"].as_str().unwrap(),
                                "content": read_tool(file_path),
                            });
                            msgs.push(tool_msg);
                        }
                    }
                    "Write" => {
                        let args: Value =
                            serde_json::from_str(function_arguments).unwrap_or_default();
                        let file_path_option = args["file_path"].as_str();
                        let content_option = args["content"].as_str();

                        if file_path_option.is_some() && content_option.is_some() {
                            let file_path = file_path_option.unwrap();
                            let content = content_option.unwrap();
                            let tool_msg = json!({
                                "role": "tool",
                                "tool_call_id": tool_call["id"].as_str().unwrap(),
                                "content": write_tool(file_path, content),
                            });
                            msgs.push(tool_msg);
                        }
                    }
                    "Bash" => {
                        let args: Value =
                            serde_json::from_str(function_arguments).unwrap_or_default();
                        if let Some(command) = args["command"].as_str() {
                            let tool_msg = json!({
                                "role": "tool",
                                "tool_call_id": tool_call["id"].as_str().unwrap(),
                                "content": bash_tool(command),
                            });
                            msgs.push(tool_msg);
                        }
                    }
                    _ => (),
                }
            }
        } else if let Some(content) = msg["content"].as_str() {
            println!("{}", content);
            break;
        }
    }

    Ok(())
}
