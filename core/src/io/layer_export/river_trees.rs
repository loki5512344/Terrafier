use crate::coords::TILE_SIZE;
use crate::io::layer_export::LayerExport;
use crate::model::layers::{LAYER_RIVER, LAYER_TREES};
use crate::model::tile::{LayerBuffer, Tile};

pub struct RiverLayerExport;
impl LayerExport for RiverLayerExport {
    fn layer_id(&self) -> u32 {
        LAYER_RIVER
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
        let data = tile.get_layer_data(LAYER_RIVER)?;
        if let LayerBuffer::Byte(bytes) = data {
            let value = bytes[idx];
            if value > 0 {
                let depth = value as i32;
                if y <= surface_y && y > surface_y - depth {
                    if y >= surface_y - 1 {
                        Some("minecraft:water")
                    } else {
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
                    }
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

pub struct TreesLayerExport;
impl LayerExport for TreesLayerExport {
    fn layer_id(&self) -> u32 {
        LAYER_TREES
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
        let data = tile.get_layer_data(LAYER_TREES)?;
        if let LayerBuffer::Byte(bytes) = data {
            let value = bytes[idx];
            if value > 0
                && block_name == "minecraft:air"
                && y > surface_y
                && y <= surface_y + value as i32
            {
                Some("minecraft:oak_log")
            } else {
                None
            }
        } else {
            None
        }
    }
}
