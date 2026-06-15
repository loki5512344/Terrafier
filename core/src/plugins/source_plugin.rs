use crate::model::tile::Tile;

/// A plugin that provides custom tile or heightmap generation.
pub trait TileSourcePlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn generate_tile(&self, tile: &mut Tile, seed: u64);
}
