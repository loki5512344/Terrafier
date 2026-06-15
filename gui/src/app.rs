use std::collections::VecDeque;

use terrafier_core::model::terrain::Terrain;
use terrafier_core::ops::operations::Operation;
use terrafier_core::World;

#[derive(Clone, Copy, PartialEq)]
pub enum ToolMode {
    Raise,
    Lower,
    Smooth,
    Flatten,
    Paint,
    Inspect,
}

impl ToolMode {
    pub fn name(&self) -> &'static str {
        match self {
            ToolMode::Raise => "Raise",
            ToolMode::Lower => "Lower",
            ToolMode::Smooth => "Smooth",
            ToolMode::Flatten => "Flatten",
            ToolMode::Paint => "Paint",
            ToolMode::Inspect => "Inspect",
        }
    }
}

pub struct TerrafierApp {
    pub world: Option<World>,
    pub selected_tile: Option<(i32, i32)>,
    pub tool_mode: ToolMode,
    pub brush_radius: u32,
    pub brush_strength: f64,
    pub selected_terrain: Terrain,
    pub target_height: i16,
    pub smooth_iterations: u32,
    pub undo_stack: VecDeque<Box<dyn Operation>>,
    pub redo_stack: Vec<Box<dyn Operation>>,
    pub brush_local_x: Option<u32>,
    pub brush_local_z: Option<u32>,
    pub show_heightmap: bool,
    pub zoom: f32,
    pub view_offset: (f32, f32),
    pub show_new_world: bool,
    pub show_export: bool,
    pub status_message: String,
    pub world_name: String,
    pub world_seed: String,
    pub export_path: String,
}

impl TerrafierApp {
    pub fn new() -> Self {
        Self {
            world: None,
            selected_tile: None,
            tool_mode: ToolMode::Raise,
            brush_radius: 16,
            brush_strength: 0.5,
            selected_terrain: Terrain::Grass,
            target_height: 64,
            smooth_iterations: 3,
            undo_stack: VecDeque::new(),
            redo_stack: Vec::new(),
            brush_local_x: None,
            brush_local_z: None,
            show_heightmap: false,
            zoom: 1.0,
            view_offset: (0.0, 0.0),
            show_new_world: false,
            show_export: false,
            status_message: "Ready".to_string(),
            world_name: String::new(),
            world_seed: String::new(),
            export_path: String::new(),
        }
    }

    pub fn save_for_undo(&mut self, op: Box<dyn Operation>) {
        self.undo_stack.push_back(op);
        if self.undo_stack.len() > 50 {
            self.undo_stack.pop_front();
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(op) = self.undo_stack.pop_back() {
            if let Some(ref mut world) = self.world {
                let dim = world.overworld_mut().expect("world has no overworld");
                let inv = op.inverse();
                if let Err(e) = inv.apply(dim) {
                    self.status_message = format!("Undo error: {:?}", e);
                    self.undo_stack.push_back(op);
                    return;
                }
                self.redo_stack.push(op);
                self.status_message = "Undo".to_string();
            }
        } else {
            self.status_message = "Nothing to undo".to_string();
        }
    }

    pub fn redo(&mut self) {
        if let Some(op) = self.redo_stack.pop() {
            if let Some(ref mut world) = self.world {
                let dim = world.overworld_mut().expect("world has no overworld");
                if let Err(e) = op.apply(dim) {
                    self.status_message = format!("Redo error: {:?}", e);
                    self.redo_stack.push(op);
                    return;
                }
                self.undo_stack.push_back(op);
                self.status_message = "Redo".to_string();
            }
        } else {
            self.status_message = "Nothing to redo".to_string();
        }
    }
}

impl eframe::App for TerrafierApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("New World").clicked() {
                    self.show_new_world = true;
                }
                if ui.button("Open").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        match terrafier_core::io::import::import(&path) {
                            Ok(world) => {
                                self.world = Some(world);
                                self.selected_tile = None;
                                self.undo_stack.clear();
                                self.redo_stack.clear();
                                self.status_message = format!("Opened world from {}", path.display());
                            }
                            Err(e) => {
                                self.status_message = format!("Open error: {}", e);
                            }
                        }
                    }
                }
                if ui.button("Export").clicked() {
                    self.show_export = true;
                }
                ui.separator();
                if ui.button("Undo").clicked() {
                    self.undo();
                }
                if ui.button("Redo").clicked() {
                    self.redo();
                }
            });
        });

        egui::SidePanel::left("tools")
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                crate::tools::show_tools_panel(ui, self);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            crate::view::show_viewport(ui, self);
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
                if let Some((tx, tz)) = self.selected_tile {
                    ui.separator();
                    ui.label(format!("Tile ({}, {})", tx, tz));
                }
                if self.world.is_some() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("World loaded");
                    });
                }
            });
        });

        crate::dialogs::handle_dialogs(self, ctx);
    }
}
