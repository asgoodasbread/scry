use std::fmt::format;

use clap::Parser;
use xcap::Monitor;

#[derive(Parser, Debug)]
#[command(version, about = "A simple CLI to capture screenshots", long_about = None)]
struct Cli{
    #[arg(short, long)]
    list: bool,

    #[arg(short, long, default_value = ".")]
    out_dir: String,

    #[arg(short, long)]
    monitor: Option<usize>
}

fn main() {
    let args = Cli::parse();

    //Fetch all monitors
    let monitors = Monitor::all().expect("Failed to get monitors");
    
    if args.list {
        println!("Available monitors:");
        for (index, monitor) in monitors.iter().enumerate() {
            let name = monitor.name().unwrap_or_else(|_| String::from("Unknown_Monitor"));
            let width = monitor.width().unwrap_or(0);
            let height = monitor.height().unwrap_or(0);

            println!("{} - {} ({}x{})", index, name, width, height);
        }
        return;
    }

    let target_monitors = match args.monitor {
        Some(index) => {
            if index < monitors.len() {
                vec![monitors.into_iter().nth(index).unwrap()]
            } else {
                eprintln!("Error: Monitor index {} is invalid. You only have {} monitors", index, monitors.len());
                return;
            }
        }
        None => monitors,
    };

    for monitor in target_monitors{
        let name = monitor.name().unwrap_or_else(|_| String::from("Unknown_monitor"));
        println!("Capturing monitor: {}...", name);

        let image = match monitor.capture_image() {
            Ok(img) => img,
            Err(err) => {
                eprintln!("Warning: Failed to capture {}: {}", name, err);
                continue;
            }
        };

        let safe_name = name.replace(&['|', '\\', ':', '/'][..], "");
        let filename = format!("{}/{}.png", args.out_dir,safe_name);

        if let Err(err) = image.save(&filename) {
            eprintln!("Warning: Failed to save {}: {}", filename, err);
        } else {
            println!("Saved screenshot to {}", filename);
        }
    }
}

