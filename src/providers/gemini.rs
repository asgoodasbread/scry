use anyhow::{Context, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::Value;

use super::send;
use crate::prompt;

pub fn ask(image_png: &[u8], api_key: &str) -> anyhow::Result<String> {
    let b64_data = STANDARD.encode(image_png);

    let template_str = include_str!("gemini.json");
    let mut payload: Value =
        serde_json::from_str(template_str).context("Failed to parse providers/gemini.json")?;

    // Gemini takes the model in the URL, not the body, so lift it back out of
    // the template before sending.
    let model = payload
        .as_object_mut()
        .and_then(|obj| obj.remove("model"))
        .and_then(|model| model.as_str().map(str::to_string))
        .context("providers/gemini.json: missing a `model` string")?;

    let parts = payload["contents"][0]["parts"]
        .as_array_mut()
        .context("providers/gemini.json: contents[0].parts must be an array")?;

    let image_part = parts
        .iter_mut()
        .find(|p| p.get("inline_data").is_some())
        .context("providers/gemini.json: no inline_data part")?;
    image_part["inline_data"]["data"] = Value::String(b64_data);

    let text_part = parts
        .iter_mut()
        .find(|p| p.get("text").is_some())
        .context("providers/gemini.json: no text part")?;
    text_part["text"] = Value::String(prompt::user_prompt().to_string());

    let url =
        format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent");

    let client = reqwest::blocking::Client::new();
    let response_json = send(
        client
            .post(url)
            .header("x-goog-api-key", api_key)
            .header("content-type", "application/json")
            .json(&payload),
    )?;

    if let Some(reason) = response_json["promptFeedback"]["blockReason"].as_str() {
        bail!("The model declined to answer this request ({reason}).");
    }

    let candidate = &response_json["candidates"][0];

    // Reasoning parts come back inline flagged as `thought`, so filter them out
    // rather than taking parts[0] and hoping it is the answer.
    let answer: String = candidate["content"]["parts"]
        .as_array()
        .context("Unexpected response shape: `candidates[0].content.parts` is not an array")?
        .iter()
        .filter(|p| p["thought"] != true)
        .filter_map(|p| p["text"].as_str())
        .collect::<Vec<_>>()
        .join("");

    if answer.trim().is_empty() {
        bail!(
            "The model returned no text (finishReason: {})",
            candidate["finishReason"]
        );
    }

    Ok(answer)
}
