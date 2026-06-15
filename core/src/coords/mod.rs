//! Coordinate system for Terrafier.
//!
//! All conversions are done through explicit functions, not inline arithmetic.

/// Block coordinates (absolute, within the Minecraft world).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockCoords {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Chunk coordinates (16x16 block columns).
/// chunk_x = block_x >> 4, chunk_z = block_z >> 4
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoords {
    pub x: i32,
    pub z: i32,
}

/// Region coordinates (32x32 chunk areas = 512x512 blocks).
/// region_x = chunk_x >> 5, region_z = chunk_z >> 5
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionCoords {
    pub x: i32,
    pub z: i32,
}

/// Terrafier Tile coordinates (128x128 blocks internally).
/// tile_x = block_x >> 7, tile_z = block_z >> 7
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileCoords {
    pub x: i32,
    pub z: i32,
}

/// Local coordinates within a tile (0..127).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalTileCoords {
    pub x: u32,
    pub z: u32,
}

/// Local coordinates within a chunk (0..15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalChunkCoords {
    pub x: u8,
    pub z: u8,
}

// === Conversions ===

impl BlockCoords {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn to_chunk(self) -> ChunkCoords {
        ChunkCoords {
            x: self.x >> 4,
            z: self.z >> 4,
        }
    }

    pub fn to_region(self) -> RegionCoords {
        RegionCoords {
            x: self.x >> 9,
            z: self.z >> 9,
        }
    }

    pub fn to_tile(self) -> TileCoords {
        TileCoords {
            x: self.x >> 7,
            z: self.z >> 7,
        }
    }
}

impl ChunkCoords {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn to_region(self) -> RegionCoords {
        RegionCoords {
            x: self.x >> 5,
            z: self.z >> 5,
        }
    }

    pub fn to_block_min(self) -> BlockCoords {
        BlockCoords {
            x: self.x << 4,
            y: i32::MIN,
            z: self.z << 4,
        }
    }

    pub fn local_in_region(self) -> LocalChunkCoords {
        LocalChunkCoords {
            x: (self.x & 31) as u8,
            z: (self.z & 31) as u8,
        }
    }
}

impl RegionCoords {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Region file name like "r.x.z.mca"
    pub fn file_name(&self) -> String {
        format!("r.{}.{}.mca", self.x, self.z)
    }
}

impl TileCoords {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn local_in_tile(self, block: BlockCoords) -> LocalTileCoords {
        LocalTileCoords {
            x: (block.x & 127) as u32,
            z: (block.z & 127) as u32,
        }
    }
}

/// Terrafier tile size constant.
pub const TILE_SIZE: usize = 128;
pub const TILE_SIZE_BITS: u32 = 7;
pub const TILE_SIZE_MASK: u32 = 127;
