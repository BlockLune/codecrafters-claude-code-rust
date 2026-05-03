use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::{env, process};

mod tool;

use tool::{get_tools, read_tool};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
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

    loop {
        let response: Value = client
            .chat()
            .create_byot(json!({
                "messages": msgs,
                "model": "anthropic/claude-haiku-4.5",
                "tools": tools
            }))
            .await?;

        let msg = (&response["choices"][0]["message"]).clone();
        msgs.push(msg.clone());

        eprintln!("DEBUG: msgs: {:#?}", msgs);

        if let Some(content) = msg["content"].as_str() {
            println!("{}", content);
            break;
        }

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
                    _ => (),
                }
            }
        }
    }

    Ok(())
}
