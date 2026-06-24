use terrafier_noise::{NoiseFn, OpenSimplex};

use crate::coords::TILE_SIZE;
use crate::model::layers::{DataSize, LAYER_CAVES};
use crate::model::tile::Tile;
use crate::model::types::Terrain;

pub struct CavesGenerator;

impl crate::ops::layers::LayerGenerator for CavesGenerator {
    fn layer_id(&self) -> u32 {
        LAYER_CAVES
    }

    fn generate(&self, tile: &mut Tile, seed: u64) {
        let seed_u32 = (seed ^ (seed >> 32)) as u32;
        let noise = OpenSimplex::new(seed_u32.wrapping_add(1));

        // Read all inputs before mutating tile
        let tile_x = tile.x;
        let tile_z = tile.z;
        let heightmap = tile.heightmap;
        let terrain = tile.terrain;

        let total = TILE_SIZE * TILE_SIZE;
        let nibble_count = total.div_ceil(2);
        let mut nibbles = vec![0u8; nibble_count];

        for lz in 0..TILE_SIZE {
            for lx in 0..TILE_SIZE {
                let idx = lz * TILE_SIZE + lx;
                let height = heightmap[idx];
                let terrain_type = terrain[idx] as usize;

                if terrain_type == Terrain::Water as usize || height <= 0 {
                    continue;
                }

                let world_x = (tile_x as f64 * TILE_SIZE as f64 + lx as f64) * 0.02;
                let world_z = (tile_z as f64 * TILE_SIZE as f64 + lz as f64) * 0.02;

                let mut max_value: u8 = 0;
                let surface_min = (height - 20).max(-60);
                let surface_max = (height - 3).max(surface_min + 1);

                for y in (surface_min..=surface_max).step_by(2) {
                    let n = noise.get([world_x, world_z, y as f64 * 0.03]);
                    if n > 0.3 {
                        let v = ((n - 0.3) * 20.0) as u8;
                        if v > max_value {
                            max_value = v;
                        }
                    }
                }

                let nibble_idx = idx / 2;
                if idx.is_multiple_of(2) {
                    nibbles[nibble_idx] = (nibbles[nibble_idx] & 0x0F) | (max_value << 4);
                } else {
                    nibbles[nibble_idx] = (nibbles[nibble_idx] & 0xF0) | max_value;
                }
            }
        }

        let buf = tile.ensure_layer(LAYER_CAVES, DataSize::Nibble);
        if let crate::model::tile::LayerBuffer::Nibble(n) = buf {
            n.copy_from_slice(&nibbles);
        }
    }
}
