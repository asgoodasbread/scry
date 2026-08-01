use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

const APP_NAME: &str = "scry";

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub api_key: String,
}

pub fn load_or_init_config() -> anyhow::Result<AppConfig> {
    let mut cfg: AppConfig =
        confy::load(APP_NAME, None).context("Failed to load or create the configuration file")?;

    if cfg.api_key.is_empty() {
        println!("It looks like this is your first run, or your API key is missing.");
        print!("Please enter your Claude API key: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("Failed to read the API key from stdin")?;

        let key = input.trim();
        if key.is_empty() {
            anyhow::bail!("No API key entered.");
        }
        cfg.api_key = key.to_string();

        confy::store(APP_NAME, None, &cfg)
            .map_err(|e| anyhow::anyhow!("Failed to save config: {}", e))?;

        println!("API key saved.\n");
    }

    Ok(cfg)
}
