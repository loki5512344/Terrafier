mod export;
mod new_world;

pub fn handle_dialogs(app: &mut crate::app::TerrafierApp, ctx: &egui::Context) {
    if app.show_new_world {
        new_world::show(app, ctx);
    }
    if app.show_export {
        export::show(app, ctx);
    }
}
