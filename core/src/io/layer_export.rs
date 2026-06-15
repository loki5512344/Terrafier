//! Layer export system — applies layer data to modify blocks during Minecraft export.

use crate::coords::TILE_SIZE;
use crate::model::layers::{
    LAYER_BIOME, LAYER_CAVES, LAYER_FROST, LAYER_RESOURCES, LAYER_RIVER, LAYER_TREES,
};
use crate::model::tile::{LayerBuffer, Tile};

/// A layer exporter knows how to modify blocks based on layer data.
pub trait LayerExport: Send + Sync {
    fn layer_id(&self) -> u32;
    fn modify_block(
        &self,
        tile: &Tile,
        local_x: usize,
        local_z: usize,
        y: i32,
        surface_y: i32,
        block_name: &str,
    ) -> Option<&'static str>;

    /// Optional biome override for this position.
    fn biome_name(&self, tile: &Tile, local_x: usize, local_z: usize) -> Option<&'static str> {
        let _ = (tile, local_x, local_z);
        None
    }
}

/// Apply all layer exporters in order. The first exporter that returns a replacement wins.
pub fn apply_layers<'a>(
    tile: &Tile,
    local_x: usize,
    local_z: usize,
    y: i32,
    surface_y: i32,
    block_name: &'a str,
    layer_exporters: &[&dyn LayerExport],
) -> &'a str {
    for exporter in layer_exporters {
        if let Some(replacement) =
            exporter.modify_block(tile, local_x, local_z, y, surface_y, block_name)
        {
            return replacement;
        }
    }
    block_name
}

/// Resolve the biome name from layer exporters, falling back to plains.
pub fn biome_name(
    tile: &Tile,
    local_x: usize,
    local_z: usize,
    layer_exporters: &[&dyn LayerExport],
) -> &'static str {
    for exporter in layer_exporters {
        if let Some(name) = exporter.biome_name(tile, local_x, local_z) {
            return name;
        }
    }
    "minecraft:plains"
}

// ---------------------------------------------------------------------------
// Built-in layer exporter implementations
// ---------------------------------------------------------------------------

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

pub struct ResourcesLayerExport;
impl LayerExport for ResourcesLayerExport {
    fn layer_id(&self) -> u32 {
        LAYER_RESOURCES
    }

    fn modify_block(
        &self,
        tile: &Tile,
        local_x: usize,
        local_z: usize,
        y: i32,
        _surface_y: i32,
        block_name: &str,
    ) -> Option<&'static str> {
        let idx = local_z * TILE_SIZE + local_x;
        let data = tile.get_layer_data(LAYER_RESOURCES)?;
        if let LayerBuffer::Nibble(nibbles) = data {
            let byte = nibbles[idx / 2];
            let value = if idx.is_multiple_of(2) {
                byte >> 4
            } else {
                byte & 0xF
            };
            if value > 0 && block_name == "minecraft:stone" && y > -60 {
                let ore = match value {
                    1 => "minecraft:coal_ore",
                    2 => "minecraft:iron_ore",
                    3 => "minecraft:copper_ore",
                    4 => "minecraft:gold_ore",
                    5 => "minecraft:redstone_ore",
                    6 => "minecraft:lapis_ore",
                    7 => "minecraft:diamond_ore",
                    8 => "minecraft:emerald_ore",
                    _ => return None,
                };
                Some(ore)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct BiomeLayerExport;
impl LayerExport for BiomeLayerExport {
    fn layer_id(&self) -> u32 {
        LAYER_BIOME
    }

    fn modify_block(
        &self,
        _tile: &Tile,
        _local_x: usize,
        _local_z: usize,
        _y: i32,
        _surface_y: i32,
        _block_name: &str,
    ) -> Option<&'static str> {
        None
    }

    fn biome_name(&self, tile: &Tile, local_x: usize, local_z: usize) -> Option<&'static str> {
        let idx = local_z * TILE_SIZE + local_x;
        let data = tile.get_layer_data(LAYER_BIOME)?;
        if let LayerBuffer::Byte(bytes) = data {
            Some(match bytes[idx] {
                0 => "minecraft:plains",
                1 => "minecraft:desert",
                2 => "minecraft:forest",
                3 => "minecraft:taiga",
                4 => "minecraft:swamp",
                5 => "minecraft:snowy_plains",
                6 => "minecraft:jungle",
                7 => "minecraft:badlands",
                8 => "minecraft:ocean",
                9 => "minecraft:river",
                10 => "minecraft:mushroom_fields",
                _ => "minecraft:plains",
            })
        } else {
            None
        }
    }
}
