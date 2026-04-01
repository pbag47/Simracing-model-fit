mod app;
mod signals;

use anyhow::Result;

fn main() -> Result<()> {
    let path = std::env::args().nth(1)
        .unwrap_or_else(|| "session.srf".to_string());

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Simracing Viewer")
            .with_inner_size([1400.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Simracing Viewer",
        native_options,
        Box::new(move |cc| Ok(Box::new(app::ViewerApp::load(cc, &path)))),
    ).map_err(|e| anyhow::anyhow!("eframe error: {e}"))
}