mod app;
mod bmp;
mod export;
mod framebuffer;
mod line;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([920.0, 760.0])
            .with_min_inner_size([560.0, 440.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Paint Rust",
        options,
        Box::new(|cc| Ok(Box::new(app::PaintApp::new(cc)))),
    )
}
