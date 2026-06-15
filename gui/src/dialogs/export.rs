pub fn show(app: &mut crate::app::TerrafierApp, ctx: &egui::Context) {
    if !app.show_export {
        return;
    }
    let mut keep_open = true;
    egui::Window::new("Export").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Output path:");
            ui.text_edit_singleline(&mut app.export_path);
        });

        ui.separator();

        if ui.button("Export as Save").clicked() {
            let Some(ref world) = app.world else {
                app.status_message = "No world to export".to_string();
                return;
            };
            let path = std::path::Path::new(&app.export_path);
            match terrafier_core::io::export::export_to_save(world, path) {
                Ok(()) => {
                    app.status_message = format!("Exported to {}", app.export_path);
                    keep_open = false;
                }
                Err(e) => {
                    app.status_message = format!("Export error: {}", e);
                }
            }
        }

        if ui.button("Render as PNG").clicked() {
            let Some(ref world) = app.world else {
                app.status_message = "No world to render".to_string();
                return;
            };
            let path = std::path::Path::new(&app.export_path);
            match terrafier_core::io::export::render_to_image(world, path, 2) {
                Ok(()) => {
                    app.status_message = format!("Rendered to {}", app.export_path);
                    keep_open = false;
                }
                Err(e) => {
                    app.status_message = format!("Render error: {}", e);
                }
            }
        }

        if ui.button("Cancel").clicked() {
            keep_open = false;
        }
    });
    if !keep_open {
        app.show_export = false;
    }
}
