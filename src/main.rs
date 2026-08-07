use clap::Parser;

mod capture;
mod config;
mod output;
mod prompt;
mod providers;
mod ocr;

use providers::Provider;

#[derive(Parser, Debug)]
#[command(version, about = "Capture a screenshot and ask an AI about it", long_about = None)]
struct Cli {
    /// List available monitors and exit
    #[arg(short, long)]
    list: bool,

    /// Index of the monitor to capture (defaults to the first)
    #[arg(short, long)]
    monitor: Option<usize>,

    /// AI provider to ask (defaults to the one set in the config file)
    #[arg(short, long, value_enum)]
    provider: Option<Provider>,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    if args.list {
        capture::list_monitors()?;
        return Ok(());
    }

    let mut cfg = config::load()?;
    let provider = args.provider.unwrap_or(cfg.provider);
    let api_key = config::api_key(&mut cfg, provider)?;

    let png = capture::capture(args.monitor)?;
    let answer = providers::ask(provider, &png, &api_key)?;
    output::render(&answer);
    Ok(())
}
