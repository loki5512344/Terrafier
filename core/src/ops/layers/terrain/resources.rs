use crate::model::layers::{DataSize, LAYER_RESOURCES};
use crate::model::tile::Tile;

pub struct ResourcesGenerator;

impl crate::ops::layers::LayerGenerator for ResourcesGenerator {
    fn layer_id(&self) -> u32 {
        LAYER_RESOURCES
    }

    fn generate(&self, tile: &mut Tile, _seed: u64) {
        // Resources layer is reserved for manual painting by the user.
        // Initialize with zeros (no ores).
        tile.ensure_layer(LAYER_RESOURCES, DataSize::Nibble);
    }
}
