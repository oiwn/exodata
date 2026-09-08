use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::{Value, json};

pub fn columns() -> Vec<String> {
    [
        "text",
        "model",
        "finish_reason",
        "prompt_tokens",
        "completion_tokens",
        "total_tokens",
        "elapsed_ms",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

pub(super) fn request_body(
    input: &str,
    system_prompt: Option<&str>,
    max_tokens: u32,
) -> Value {
    let mut messages = Vec::new();
    if let Some(prompt) = system_prompt {
        messages.push(json!({"role": "system", "content": prompt}));
    }
    messages.push(json!({"role": "user", "content": input}));
    json!({
        "model": super::client::MODEL,
        "messages": messages,
        "max_tokens": max_tokens,
        "thinking": {"type": "disabled"},
        "stream": false
    })
}

pub fn run() -> Result<Value> {
    generate("Reply with only OK.", 32)
}

pub fn generate(input: &str, max_tokens: u32) -> Result<Value> {
    generate_with_system(input, None, max_tokens)
}

pub fn generate_with_system(
    input: &str,
    system_prompt: Option<&str>,
    max_tokens: u32,
) -> Result<Value> {
    if system_prompt.is_some_and(|prompt| prompt.trim().is_empty()) {
        bail!("System prompt must not be blank");
    }
    if input.trim().is_empty() {
        bail!("Generation input must not be blank");
    }
    if max_tokens == 0 {
        bail!("max_tokens must be greater than zero");
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let outcome = super::client::Client::from_env()?
            .generate(input, system_prompt, max_tokens)
            .await;
        outcome.report.ok_or_else(|| {
            anyhow::anyhow!(
                outcome
                    .error
                    .unwrap_or_else(|| "DeepSeek generation failed".into())
            )
        })
    })
}

#[derive(Deserialize)]
pub(super) struct Response {
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
    finish_reason: String,
}

#[derive(Deserialize)]
struct Message {
    content: Option<String>,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
}

pub(super) fn report(response: Response, elapsed_ms: u128) -> Result<Value> {
    let choice = response
        .choices
        .first()
        .context("DeepSeek returned no choices")?;
    if choice.finish_reason != "stop" {
        bail!(
            "DeepSeek did not finish normally (finish_reason: {}); no retry attempted",
            choice.finish_reason
        );
    }
    let text = choice.message.content.as_deref().unwrap_or_default();
    if text.trim().is_empty() {
        bail!("DeepSeek returned empty text");
    }
    Ok(json!({
        "text": text,
        "model": response.model,
        "finish_reason": choice.finish_reason,
        "prompt_tokens": response.usage.prompt_tokens,
        "completion_tokens": response.usage.completion_tokens,
        "total_tokens": response.usage.total_tokens,
        "elapsed_ms": elapsed_ms
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_arbitrary_input_and_requested_budget() {
        let input = "[instructions]\ntask = \"Write a paragraph.\"\n";
        let body = request_body(input, None, 256);
        assert_eq!(body["messages"][0]["content"], input);
        assert_eq!(body["max_tokens"], 256);
        assert_eq!(body["thinking"]["type"], "disabled");
        assert_eq!(body["stream"], false);
    }

    #[test]
    fn separates_system_instructions_from_user_evidence() {
        let body = request_body(
            "[system]\nname = 'example'",
            Some("Use only evidence.\n"),
            512,
        );
        assert_eq!(
            body["messages"],
            json!([
                {"role": "system", "content": "Use only evidence.\n"},
                {"role": "user", "content": "[system]\nname = 'example'"}
            ])
        );
        assert!(
            generate_with_system("Hello", Some(" \n"), 512)
                .unwrap_err()
                .to_string()
                .contains("System prompt must not be blank")
        );
    }

    #[test]
    fn rejects_invalid_input_before_network_access() {
        assert!(
            generate(" \n", 256)
                .unwrap_err()
                .to_string()
                .contains("blank")
        );
        assert!(
            generate("Hello", 0)
                .unwrap_err()
                .to_string()
                .contains("greater than zero")
        );
    }

    fn response(content: Value, finish_reason: &str) -> Value {
        json!({"model": "test-model", "choices": [{"message": {"content": content},
            "finish_reason": finish_reason}], "usage": {
                "prompt_tokens": 8, "completion_tokens": 1, "total_tokens": 9}})
    }

    #[test]
    fn reports_provider_usage_and_text() {
        let result = report(
            serde_json::from_value(response(json!("OK"), "stop")).unwrap(),
            25,
        )
        .unwrap();
        assert_eq!(result["text"], "OK");
        assert_eq!(result["total_tokens"], 9);
        assert_eq!(result["elapsed_ms"], 25);
    }

    #[test]
    fn rejects_empty_truncated_and_malformed_responses() {
        for content in [Value::Null, json!(""), json!("  ")] {
            assert!(
                report(
                    serde_json::from_value(response(content, "stop")).unwrap(),
                    0
                )
                .is_err()
            );
        }
        assert!(
            report(
                serde_json::from_value(response(json!("O"), "length")).unwrap(),
                0
            )
            .is_err()
        );
        let mut empty = response(json!("OK"), "stop");
        empty["choices"] = json!([]);
        assert!(report(serde_json::from_value(empty).unwrap(), 0).is_err());
        assert!(serde_json::from_value::<Response>(json!({})).is_err());
    }
}
