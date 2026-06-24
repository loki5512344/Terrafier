use crate::coords::TILE_SIZE;
use crate::model::layers::{DataSize, LAYER_BIOME};
use crate::model::tile::Tile;
use crate::ops::layers::LayerGenerator;

pub struct BiomeGenerator;

impl LayerGenerator for BiomeGenerator {
    fn layer_id(&self) -> u32 {
        LAYER_BIOME
    }

    fn generate(&self, tile: &mut Tile, _seed: u64) {
        let data = from_terrain(&tile.terrain);
        let buf = tile.ensure_layer(LAYER_BIOME, DataSize::Byte);
        if let crate::model::tile::LayerBuffer::Byte(bytes) = buf {
            bytes.copy_from_slice(&data);
        }
    }
}

/// Map terrain types to biome indices used by BiomeLayerExport.
/// Biome index → MC biome ID mapping (from biome_res.rs):
///   0 → 1 (Plains),  1 → 2 (Desert),  2 → 4 (Forest),
///   3 → 5 (Taiga),   4 → 6 (Swamp),   8 → 0 (Ocean)
const TERRAIN_TO_BIOME: [u8; 7] = [
    0, // Desert → Plains (index 0)
    0, // Grass  → Plains (index 0)
    2, // Forest → Forest (index 2)
    3, // Rock   → Taiga  (index 3)
    1, // Sand   → Desert (index 1)
    4, // Swamp  → Swamp  (index 4)
    8, // Water  → Ocean  (index 8)
];

pub fn from_terrain(terrain: &[u8]) -> Vec<u8> {
    terrain
        .iter()
        .map(|&t| {
            let idx = t as usize;
            if idx < TERRAIN_TO_BIOME.len() {
                TERRAIN_TO_BIOME[idx]
            } else {
                0
            }
        })
        .collect()
}

#[allow(dead_code)]
pub fn biome_at(tile: &Tile, local_x: usize, local_z: usize) -> u8 {
    let idx = local_z * TILE_SIZE + local_x;
    let terrain = tile.terrain[idx] as usize;
    if terrain < TERRAIN_TO_BIOME.len() {
        TERRAIN_TO_BIOME[terrain]
    } else {
        0
    }
}
