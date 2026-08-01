use anyhow::Context;
use std::io::Cursor;
use xcap::Monitor;
use xcap::image::ImageFormat;

pub fn list_monitors() -> anyhow::Result<()> {
    let monitors = Monitor::all().context("Failed to enumerate monitors")?;

    println!("Available monitors:");
    for (index, monitor) in monitors.iter().enumerate() {
        let name = monitor
            .name()
            .unwrap_or_else(|_| String::from("Unknown monitor"));
        let width = monitor.width().unwrap_or(0);
        let height = monitor.height().unwrap_or(0);

        println!("  {index} - {name} ({width}x{height})");
    }

    Ok(())
}

pub fn capture(monitor_index: Option<usize>) -> anyhow::Result<Vec<u8>> {
    let monitors = Monitor::all().context("Failed to enumerate monitors")?;

    let monitor = match monitor_index {
        Some(i) => monitors.into_iter().nth(i).with_context(|| {
            format!("Invalid monitor index {i} — run `scry --list` to see available monitors")
        })?,
        None => monitors
            .into_iter()
            .next()
            .context("No monitors found")?,
    };

    let image = monitor
        .capture_image()
        .context("Failed to capture the screen")?;

    let mut buf = Cursor::new(Vec::new());
    image
        .write_to(&mut buf, ImageFormat::Png)
        .context("Failed to encode the screenshot as PNG")?;

    Ok(buf.into_inner())
}
