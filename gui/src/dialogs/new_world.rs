use terrafier_core::World;

pub fn show(app: &mut crate::app::TerrafierApp, ctx: &egui::Context) {
    if !app.show_new_world {
        return;
    }
    let mut keep_open = true;
    egui::Window::new("New World").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut app.world_name);
        });

        ui.horizontal(|ui| {
            ui.label("Seed:");
            ui.text_edit_singleline(&mut app.world_seed);
        });

        ui.separator();

        if ui.button("Create").clicked() {
            let name = if app.world_name.is_empty() {
                "World"
            } else {
                &app.world_name
            };
            let seed = app.world_seed.parse::<u64>().unwrap_or(0);
            app.world = Some(World::new(name, seed));
            app.selected_tile = None;
            app.undo_stack.clear();
            app.redo_stack.clear();
            app.status_message = format!("Created world '{}' with seed {}", name, seed);
            keep_open = false;
        }

        if ui.button("Cancel").clicked() {
            keep_open = false;
        }
    });
    if !keep_open {
        app.show_new_world = false;
    }
}
