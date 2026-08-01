use anyhow::{Context, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::Value;

use crate::prompt;

pub fn ask(image_png: &[u8], api_key: &str) -> anyhow::Result<String> {
    let b64_data = STANDARD.encode(image_png);

    let template_str = include_str!("payload_template.json");
    let mut payload: Value =
        serde_json::from_str(template_str).context("Failed to parse payload_template.json")?;

    // Locate the blocks by their `type` rather than by fixed index, so reordering
    // the template can't silently write base64 into the text block.
    let content = payload["messages"][0]["content"]
        .as_array_mut()
        .context("payload_template.json: messages[0].content must be an array")?;

    let image_block = content
        .iter_mut()
        .find(|b| b["type"] == "image")
        .context("payload_template.json: no image content block")?;
    image_block["source"]["data"] = Value::String(b64_data);

    let text_block = content
        .iter_mut()
        .find(|b| b["type"] == "text")
        .context("payload_template.json: no text content block")?;
    text_block["text"] = Value::String(prompt::user_prompt().to_string());

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&payload)
        .send()
        .context("Failed to send request to Anthropic API")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().unwrap_or_default();
        bail!("API error ({status}): {error_text}");
    }

    let response_json: Value = response.json().context("Failed to parse JSON response")?;

    if response_json["stop_reason"] == "refusal" {
        bail!("Claude declined to answer this request.");
    }

    // Thinking is on by default, so content[0] is not necessarily the answer —
    // collect every text block rather than indexing into the first one.
    let answer: String = response_json["content"]
        .as_array()
        .context("Unexpected response shape: `content` is not an array")?
        .iter()
        .filter(|b| b["type"] == "text")
        .filter_map(|b| b["text"].as_str())
        .collect::<Vec<_>>()
        .join("");

    if answer.trim().is_empty() {
        bail!(
            "Claude returned no text (stop_reason: {})",
            response_json["stop_reason"]
        );
    }

    Ok(answer)
}
