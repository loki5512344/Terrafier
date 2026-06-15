use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

pub use crate::coords::{TILE_SIZE, TILE_SIZE_BITS};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub x: i32,
    pub z: i32,
    pub min_height: i16,
    pub max_height: i16,
    #[serde(with = "BigArray")]
    pub heightmap: [i16; 16384],
    #[serde(with = "BigArray")]
    pub terrain: [u8; 16384],
    #[serde(with = "BigArray")]
    pub water_level: [u8; 16384],
    pub layer_data: std::collections::HashMap<u32, LayerBuffer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerBuffer {
    Bit(Vec<u64>),
    Nibble(Vec<u8>),
    Byte(Vec<u8>),
    Int(Vec<i32>),
}

impl Tile {
    pub fn new(x: i32, z: i32, min_height: i16, max_height: i16) -> Self {
        Self {
            x,
            z,
            min_height,
            max_height,
            heightmap: [0i16; 16384],
            terrain: [0u8; 16384],
            water_level: [0u8; 16384],
            layer_data: std::collections::HashMap::new(),
        }
    }

    /// Get height at local tile coordinates.
    pub fn get_height(&self, lx: usize, lz: usize) -> i16 {
        self.heightmap[lz * TILE_SIZE + lx]
    }

    /// Set height at local tile coordinates.
    pub fn set_height(&mut self, lx: usize, lz: usize, height: i16) {
        self.heightmap[lz * TILE_SIZE + lx] = height;
    }

    /// Get terrain type at local tile coordinates.
    pub fn get_terrain(&self, lx: usize, lz: usize) -> u8 {
        self.terrain[lz * TILE_SIZE + lx]
    }

    /// Set terrain type at local tile coordinates.
    pub fn set_terrain(&mut self, lx: usize, lz: usize, terrain: u8) {
        self.terrain[lz * TILE_SIZE + lx] = terrain;
    }
}
