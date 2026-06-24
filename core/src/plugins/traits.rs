use crate::model::layers::Layer;
use crate::model::tile::Tile;
use crate::model::world::World;
use crate::ops::operations::Operation;
use std::path::Path;

pub trait ExportPlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn can_export(&self, world: &World) -> bool;
    fn export(&self, world: &World, path: &Path) -> Result<(), Box<dyn std::error::Error>>;
    fn format_name(&self) -> &'static str;
}

pub trait LayerPlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn layers(&self) -> Vec<Box<dyn Layer>>;
}

pub trait OperationPlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn operations(&self) -> Vec<Box<dyn Fn() -> Box<dyn Operation>>>;
}

pub trait TileSourcePlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn generate_tile(&self, tile: &mut Tile, seed: u64);
}
