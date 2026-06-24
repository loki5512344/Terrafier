use crate::coords::TILE_SIZE;
use crate::model::layers::{DataSize, LAYER_FROST};
use crate::model::tile::Tile;

const SNOW_LINE: i16 = 90;

pub struct FrostGenerator;

impl crate::ops::layers::LayerGenerator for FrostGenerator {
    fn layer_id(&self) -> u32 {
        LAYER_FROST
    }

    fn generate(&self, tile: &mut Tile, _seed: u64) {
        let heightmap = tile.heightmap;

        let total = TILE_SIZE * TILE_SIZE;
        let word_count = total.div_ceil(64);
        let mut bits = vec![0u64; word_count];

        for lz in 0..TILE_SIZE {
            for lx in 0..TILE_SIZE {
                let idx = lz * TILE_SIZE + lx;
                if heightmap[idx] >= SNOW_LINE {
                    let word_idx = idx / 64;
                    let bit_idx = idx % 64;
                    bits[word_idx] |= 1u64 << bit_idx;
                }
            }
        }

        let buf = tile.ensure_layer(LAYER_FROST, DataSize::Bit);
        if let crate::model::tile::LayerBuffer::Bit(b) = buf {
            b.copy_from_slice(&bits);
        }
    }
}
