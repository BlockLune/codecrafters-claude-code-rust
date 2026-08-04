mod message;
mod skill;
mod tool;

use anyhow::{Context, Result};
use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::env;
use tool::get_tools;

use crate::message::build_system_prompt;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
    #[arg(long)]
    model: Option<String>,
    #[arg(long)]
    append_system_prompt: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let base_url =
        env::var("OPENROUTER_BASE_URL").unwrap_or("https://openrouter.ai/api/v1".to_string());
    let api_key = env::var("OPENROUTER_API_KEY").context("OPENROUTER_API_KEY is not set")?;

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);
    let tools = get_tools();

    let system_prompt = build_system_prompt(args.append_system_prompt.as_deref())?;
    let system_msg = json!({ "role": "system", "content": system_prompt});
    let first_user_msg = json!({ "role": "user", "content": args.prompt });
    let mut msgs = vec![system_msg, first_user_msg];

    let model = args
        .model
        .unwrap_or("anthropic/claude-haiku-4.5".to_string());

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
                let Some(tool_call_id) = tool_call["id"].as_str() else {
                    continue;
                };
                let Some(function_name) = tool_call["function"]["name"].as_str() else {
                    continue;
                };
                let Some(function_arguments) = tool_call["function"]["arguments"].as_str() else {
                    continue;
                };
                let tool_msg = tool::execute(tool_call_id, function_name, &function_arguments)?;
                msgs.push(tool_msg);
            }
        } else if let Some(content) = msg["content"].as_str() {
            println!("{}", content);
            break;
        }
    }

    Ok(())
}
