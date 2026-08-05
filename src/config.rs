use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

use crate::providers::Provider;

const APP_NAME: &str = "scry";

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub provider: Provider,

    /// Superseded by `keys`; migrated to `keys.anthropic` on load.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub api_key: String,

    #[serde(default)]
    pub keys: Keys,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Keys {
    #[serde(default)]
    pub anthropic: String,
    #[serde(default)]
    pub gemini: String,
}

impl Keys {
    fn get(&self, provider: Provider) -> &str {
        match provider {
            Provider::Anthropic => &self.anthropic,
            Provider::Gemini => &self.gemini,
        }
    }

    fn set(&mut self, provider: Provider, key: String) {
        match provider {
            Provider::Anthropic => self.anthropic = key,
            Provider::Gemini => self.gemini = key,
        }
    }
}

pub fn load() -> anyhow::Result<AppConfig> {
    confy::load(APP_NAME, None).context("Failed to load or create the configuration file")
}

/// Resolves the key for `provider`, preferring the environment over the config
/// file, and prompting for it as a last resort.
pub fn api_key(cfg: &mut AppConfig, provider: Provider) -> anyhow::Result<String> {
    if let Ok(key) = std::env::var(provider.env_var()) {
        let key = key.trim();
        if !key.is_empty() {
            return Ok(key.to_string());
        }
    }

    let mut dirty = false;

    if !cfg.api_key.is_empty() && cfg.keys.anthropic.is_empty() {
        cfg.keys.anthropic = std::mem::take(&mut cfg.api_key);
        dirty = true;
    }

    if cfg.keys.get(provider).is_empty() {
        cfg.keys.set(provider, prompt_for_key(provider)?);
        dirty = true;
    }

    if dirty {
        confy::store(APP_NAME, None, &cfg)
            .map_err(|e| anyhow::anyhow!("Failed to save config: {}", e))?;
    }

    Ok(cfg.keys.get(provider).to_string())
}

fn prompt_for_key(provider: Provider) -> anyhow::Result<String> {
    println!(
        "No {} API key found. Set {} in your environment, or enter it below to save it.",
        provider.label(),
        provider.env_var()
    );
    print!("{} API key: ", provider.label());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read the API key from stdin")?;

    let key = input.trim();
    if key.is_empty() {
        anyhow::bail!("No API key entered.");
    }

    println!("API key saved.\n");
    Ok(key.to_string())
}
