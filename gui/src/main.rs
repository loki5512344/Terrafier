fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Terrafier",
        native_options,
        Box::new(|_cc| Ok(Box::new(app::TerrafierApp::new()))),
    )?;
    Ok(())
}

mod app;
mod dialogs;
mod tools;
mod view;
