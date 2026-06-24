mod cpu;
mod gpu;
mod tools;

pub use tools::show_tools_panel;

pub fn show_viewport(ui: &mut egui::Ui, app: &mut crate::app::TerrafierApp) {
    if app.world.is_none() {
        ui.centered_and_justified(|ui| {
            ui.heading("No world loaded. Click 'New World' to begin.");
        });
        return;
    }

    if gpu::try_render(ui, app) {
        return;
    }
    cpu::render(ui, app);
}
