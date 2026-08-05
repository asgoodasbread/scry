use anyhow::{Context, bail};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod anthropic;
mod gemini;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
#[value(rename_all = "lowercase")]
pub enum Provider {
    #[default]
    Anthropic,
    Gemini,
}

impl Provider {
    pub fn label(self) -> &'static str {
        match self {
            Provider::Anthropic => "Anthropic",
            Provider::Gemini => "Google Gemini",
        }
    }

    pub fn env_var(self) -> &'static str {
        match self {
            Provider::Anthropic => "ANTHROPIC_API_KEY",
            Provider::Gemini => "GEMINI_API_KEY",
        }
    }
}

pub fn ask(provider: Provider, image_png: &[u8], api_key: &str) -> anyhow::Result<String> {
    match provider {
        Provider::Anthropic => anthropic::ask(image_png, api_key),
        Provider::Gemini => gemini::ask(image_png, api_key),
    }
}

fn send(request: reqwest::blocking::RequestBuilder) -> anyhow::Result<Value> {
    let response = request.send().context("Failed to send request")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().unwrap_or_default();
        bail!("API error ({status}): {error_text}");
    }

    response.json().context("Failed to parse JSON response")
}
