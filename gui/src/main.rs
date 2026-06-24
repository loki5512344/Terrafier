fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Terrafier",
        native_options,
        Box::new(|cc| Ok(Box::new(terrafier_gui::TerrafierApp::new(cc)))),
    )
}
