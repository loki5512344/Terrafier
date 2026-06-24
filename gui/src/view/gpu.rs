use egui::{Color32, Vec2};
use terrafier_core::model::tile::TILE_SIZE;

pub fn try_render(ui: &mut egui::Ui, app: &mut crate::app::TerrafierApp) -> bool {
    let Some(ref world) = app.world else {
        return false;
    };
    let dim = &world.dimensions[0];

    if let Some(ref renderer) = app.renderer
        && let Some(texture_id) = renderer.texture_id
    {
        let desired_size = Vec2::new(512.0, 512.0);
        let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::click());
        let rect = response.rect;
        painter.image(
            texture_id,
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        let display_size = rect.width() / 4.0;

        if let Some((sel_tx, sel_tz)) = app.selected_tile {
            let mut min_tx = i32::MAX;
            let mut max_tx = i32::MIN;
            let mut min_tz = i32::MAX;
            let mut max_tz = i32::MIN;
            for &(tx, tz) in dim.tiles.keys() {
                min_tx = min_tx.min(tx);
                max_tx = max_tx.max(tx);
                min_tz = min_tz.min(tz);
                max_tz = max_tz.max(tz);
            }

            let bx = rect.min.x;
            let bz = rect.min.y;
            let sel_bx = bx + (sel_tx - min_tx) as f32 * display_size;
            let sel_bz = bz + (sel_tz - min_tz) as f32 * display_size;
            let border_rect = egui::Rect::from_min_size(
                egui::pos2(sel_bx, sel_bz),
                Vec2::splat(display_size),
            );
            painter.rect_stroke(
                border_rect,
                0.0,
                egui::Stroke::new(3.0, Color32::WHITE),
                egui::StrokeKind::Middle,
            );
        }

        if let (Some((sel_tx, sel_tz)), Some(bx), Some(bz)) =
            (app.selected_tile, app.brush_local_x, app.brush_local_z)
        {
            let mut min_tx = i32::MAX;
            let mut min_tz = i32::MAX;
            for &(tx, tz) in dim.tiles.keys() {
                min_tx = min_tx.min(tx);
                min_tz = min_tz.min(tz);
            }
            let tile_x_in_pixels = (sel_tx - min_tx) as f32 * display_size;
            let tile_z_in_pixels = (sel_tz - min_tz) as f32 * display_size;
            let brush_x = rect.min.x + tile_x_in_pixels
                + (bx as f32 * display_size / TILE_SIZE as f32);
            let brush_z = rect.min.y + tile_z_in_pixels
                + (bz as f32 * display_size / TILE_SIZE as f32);
            let brush_radius_px = app.brush_radius as f32 * display_size / TILE_SIZE as f32;
            painter.circle_stroke(
                egui::pos2(brush_x, brush_z),
                brush_radius_px.max(2.0),
                egui::Stroke::new(1.5, Color32::YELLOW),
            );
        }

        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            let lx = pos.x - rect.min.x;
            let lz = pos.y - rect.min.y;
            if lx >= 0.0 && lz >= 0.0 {
                let mut min_tx = i32::MAX;
                let mut min_tz = i32::MAX;
                for &(tx, tz) in dim.tiles.keys() {
                    min_tx = min_tx.min(tx);
                    min_tz = min_tz.min(tz);
                }
                let tx = (lx / display_size).floor() as i32 + min_tx;
                let tz = (lz / display_size).floor() as i32 + min_tz;
                if dim.tiles.contains_key(&(tx, tz)) {
                    app.selected_tile = Some((tx, tz));
                    let local_x =
                        ((lx as u32 % display_size as u32) * TILE_SIZE as u32 / display_size as u32).min(127);
                    let local_z =
                        ((lz as u32 % display_size as u32) * TILE_SIZE as u32 / display_size as u32).min(127);
                    app.brush_local_x = Some(local_x);
                    app.brush_local_z = Some(local_z);
                    app.status_message = format!(
                        "Selected tile ({}, {}) at local ({}, {})",
                        tx, tz, local_x, local_z
                    );
                }
            }
        }

        return true;
    }
    false
}
