use crate::coords::TILE_SIZE;
use crate::io::layer_export::LayerExport;
use crate::model::layers::{LAYER_CAVES, LAYER_FROST};
use crate::model::tile::{LayerBuffer, Tile};

pub struct CavesLayerExport;
impl LayerExport for CavesLayerExport {
    fn layer_id(&self) -> u32 {
        LAYER_CAVES
    }

    fn modify_block(
        &self,
        tile: &Tile,
        local_x: usize,
        local_z: usize,
        y: i32,
        surface_y: i32,
        block_name: &str,
    ) -> Option<&'static str> {
        let idx = local_z * TILE_SIZE + local_x;
        let data = tile.get_layer_data(LAYER_CAVES)?;
        if let LayerBuffer::Nibble(nibbles) = data {
            let byte = nibbles[idx / 2];
            let value = if idx.is_multiple_of(2) {
                byte >> 4
            } else {
                byte & 0xF
            };
            if value > 0 && y < surface_y && y >= surface_y - 12 && y > -60 {
                match block_name {
                    "minecraft:stone"
                    | "minecraft:deepslate"
                    | "minecraft:dirt"
                    | "minecraft:grass_block"
                    | "minecraft:sand"
                    | "minecraft:sandstone"
                    | "minecraft:gravel" => Some("minecraft:air"),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct FrostLayerExport;
impl LayerExport for FrostLayerExport {
    fn layer_id(&self) -> u32 {
        LAYER_FROST
    }

    fn modify_block(
        &self,
        tile: &Tile,
        local_x: usize,
        local_z: usize,
        y: i32,
        surface_y: i32,
        _block_name: &str,
    ) -> Option<&'static str> {
        let idx = local_z * TILE_SIZE + local_x;
        let data = tile.get_layer_data(LAYER_FROST)?;
        if let LayerBuffer::Bit(bits) = data {
            let word = bits[idx / 64];
            let bit = (word >> (idx % 64)) & 1;
            if bit == 1 && y == surface_y {
                if _block_name == "minecraft:water" {
                    Some("minecraft:ice")
                } else if _block_name == "minecraft:grass_block" {
                    Some("minecraft:snow_block")
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}
