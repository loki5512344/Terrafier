use terrafier_noise::{NoiseFn, OpenSimplex};

use crate::coords::TILE_SIZE;
use crate::model::layers::{DataSize, LAYER_RIVER};
use crate::model::tile::Tile;
use crate::model::types::Terrain;

pub struct RiverGenerator;

impl crate::ops::layers::LayerGenerator for RiverGenerator {
    fn layer_id(&self) -> u32 {
        LAYER_RIVER
    }

    fn generate(&self, tile: &mut Tile, seed: u64) {
        let seed_u32 = (seed ^ (seed >> 32)) as u32;
        let noise = OpenSimplex::new(seed_u32.wrapping_add(2));

        // Read all inputs before mutating tile
        let tile_x = tile.x;
        let tile_z = tile.z;
        let heightmap = tile.heightmap;
        let terrain = tile.terrain;

        let total = TILE_SIZE * TILE_SIZE;
        let mut bytes = vec![0u8; total];

        let tile_wx = tile_x as f64 * TILE_SIZE as f64;
        let tile_wz = tile_z as f64 * TILE_SIZE as f64;

        for lz in 0..TILE_SIZE {
            for lx in 0..TILE_SIZE {
                let idx = lz * TILE_SIZE + lx;
                let height = heightmap[idx];
                let terrain_type = terrain[idx] as usize;

                if terrain_type == Terrain::Water as usize || height <= 0 {
                    continue;
                }

                let world_x = tile_wx + lx as f64;
                let world_z = tile_wz + lz as f64;

                let base_offset = noise.get([world_x * 0.005, world_z * 0.005]) * 15.0;
                let base_height = 55.0 + base_offset;
                let river_noise = noise.get([world_x * 0.02 + 100.0, world_z * 0.02 + 100.0]);

                if (height as f64) < base_height && river_noise > 0.1 {
                    let depth = (base_height - height as f64).clamp(1.0, 8.0) as u8;
                    bytes[idx] = depth;
                } else {
                    bytes[idx] = 0;
                }
            }
        }

        let buf = tile.ensure_layer(LAYER_RIVER, DataSize::Byte);
        if let crate::model::tile::LayerBuffer::Byte(b) = buf {
            b.copy_from_slice(&bytes);
        }
    }
}
