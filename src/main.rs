use clap::Parser;

mod ai;
mod capture;
mod config;
mod output;
mod prompt;

#[derive(Parser, Debug)]
#[command(version, about = "Capture a screenshot and ask an AI about it", long_about = None)]
struct Cli {
    /// List available monitors and exit
    #[arg(short, long)]
    list: bool,

    /// Index of the monitor to capture (defaults to the first)
    #[arg(short, long)]
    monitor: Option<usize>,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    if args.list {
        capture::list_monitors()?;
        return Ok(());
    }

    let cfg = config::load_or_init_config()?;
    let png = capture::capture(args.monitor)?;
    let answer = ai::ask(&png, &cfg.api_key)?;
    output::render(&answer);
    Ok(())
}
