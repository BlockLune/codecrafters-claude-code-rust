mod message;
mod skill;
mod tool;

use crate::message::build_system_prompt;
use anyhow::{Context, Result};
use async_openai::{Client, config::OpenAIConfig, types::chat::CreateChatCompletionStreamResponse};
use clap::Parser;
use futures_util::StreamExt;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    env,
    io::{Write, stdout},
};
use tool::get_tools;

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
        let mut stream = client
            .chat()
            .create_stream_byot::<_, CreateChatCompletionStreamResponse>(json!({
                "messages": msgs,
                "model": model,
                "tools": tools,
                "stream": true,
            }))
            .await?;

        let mut msg = String::new();
        let mut tools: HashMap<u32, serde_json::Value> = HashMap::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;

            let Some(choice) = chunk.choices.first() else {
                continue;
            };

            // normal message content
            if let Some(content) = &choice.delta.content {
                print!("{}", content);
                stdout().flush()?;
                msg.push_str(content);
            }

            // tool calls
            if let Some(tool_calls) = &choice.delta.tool_calls {
                for tool_call in tool_calls {
                    let tool = tools
                        .entry(tool_call.index)
                        .or_insert(serde_json::Value::default());
                    if let Some(tool_call_id) = &tool_call.id {
                        tool["id"] = json!(tool_call_id);
                    }
                    if let Some(function) = &tool_call.function {
                        if let Some(function_name) = &function.name {
                            tool["name"] = json!(function_name);
                        }
                        if let Some(function_arguments) = &function.arguments {
                            tool["arguments"] = json!(format!(
                                "{}{}",
                                tool.get("arguments")
                                    .unwrap_or_default()
                                    .as_str()
                                    .unwrap_or_default(),
                                function_arguments
                            ));
                        }
                    }
                }
            }
        }

        let mut accumulated_calls: Vec<_> = tools.into_iter().collect();
        accumulated_calls.sort_by_key(|(index, _)| *index);

        if accumulated_calls.is_empty() {
            msgs.push(json!({ "role": "assistant", "content": msg }));
            println!();
            break;
        }

        let tool_calls: Vec<Value> = accumulated_calls
            .iter()
            .map(|(_, tool)| {
                json!({
                    "id": tool["id"].clone(),
                    "type": "function",
                    "function": {
                        "name": tool["name"].clone(),
                        "arguments": tool["arguments"].clone(),
                    }
                })
            })
            .collect();

        msgs.push(json!({
            "role": "assistant",
            "content": if msg.is_empty() { Value::Null } else { Value::String(msg) },
            "tool_calls": tool_calls,
        }));

        for (_, tool) in accumulated_calls {
            let (Some(tool_id), Some(function_name), Some(function_arguments)) = (
                tool["id"].as_str(),
                tool["name"].as_str(),
                tool["arguments"].as_str(),
            ) else {
                continue;
            };
            msgs.push(tool::execute(tool_id, function_name, function_arguments)?);
        }
    }
    Ok(())
}
