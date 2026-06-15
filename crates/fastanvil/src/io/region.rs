//! Region file format (.mca / .mcr).
//!
//! A region file contains 32x32 chunks stored in a 8KB header
//! (2KB location table + 2KB timestamp table + 4KB padding)
//! followed by chunk data sectors of 4KB each.

use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};
use thiserror::Error;

use crate::compression::{self, CompressionType};

#[derive(Error, Debug)]
pub enum RegionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Compression error: {0}")]
    Compression(#[from] compression::CompressionError),
    #[error("Invalid region header at offset {0}")]
    InvalidHeader(u32),
    #[error("Chunk ({0}, {1}) not found in region")]
    ChunkNotFound(i32, i32),
}

pub type Result<T> = std::result::Result<T, RegionError>;

/// A region file containing up to 32x32 chunks.
pub struct Region {
    pub x: i32,
    pub z: i32,
    chunks: HashMap<(u8, u8), ChunkEntry>,
}

pub struct ChunkEntry {
    pub offset: u32,
    pub size: u32,
    pub timestamp: u32,
    pub data: Option<Vec<u8>>,
}

impl Region {
    /// Open a region file from raw bytes.
    pub fn from_bytes(x: i32, z: i32, data: &[u8]) -> Result<Self> {
        let mut reader = Cursor::new(data);
        let mut locations = [0u32; 1024];
        let mut timestamps = [0u32; 1024];

        // Read location table (first 4096 bytes: 1024 entries x 4 bytes)
        for i in 0..1024 {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            locations[i] = u32::from_be_bytes(buf);
        }

        // Read timestamp table (second 4096 bytes: 1024 entries x 4 bytes)
        for i in 0..1024 {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            timestamps[i] = u32::from_be_bytes(buf);
        }

        let mut chunks = HashMap::new();

        for i in 0..1024 {
            let loc = locations[i];
            if loc == 0 {
                continue;
            }
            let sector_offset = loc >> 8;
            let sector_count = loc & 0xFF;
            let timestamp = timestamps[i];

            if sector_offset == 0 {
                continue;
            }

            // Read chunk header: 4 bytes length (including 1 byte compression type)
            let byte_offset = (sector_offset as u64) * 4096;
            reader.seek(SeekFrom::Start(byte_offset))?;
            let mut len_buf = [0u8; 4];
            reader.read_exact(&mut len_buf)?;
            let chunk_data_len = u32::from_be_bytes(len_buf);

            // Compression type byte follows the length
            let mut comp_type_buf = [0u8; 1];
            reader.read_exact(&mut comp_type_buf)?;
            let compression_scheme = comp_type_buf[0];

            // Read compressed chunk payload
            let payload_len = if chunk_data_len > 0 {
                chunk_data_len as usize - 1
            } else {
                0
            };
            let mut compressed = vec![0u8; payload_len];
            reader.read_exact(&mut compressed)?;

            // Determine compression type
            let scheme = match CompressionType::from_id(compression_scheme) {
                Some(s) => s,
                None => continue,
            };

            // Decompress
            let decompressed = compression::decompress(&compressed, scheme)?;

            let local_x = (i % 32) as u8;
            let local_z = (i / 32) as u8;

            chunks.insert(
                (local_x, local_z),
                ChunkEntry {
                    offset: sector_offset,
                    size: sector_count as u32,
                    timestamp,
                    data: Some(decompressed),
                },
            );
        }

        Ok(Self { x, z, chunks })
    }

    /// Get the decompressed NBT data for a chunk at local coordinates (0..32).
    pub fn get_chunk_data(&self, local_x: u8, local_z: u8) -> Option<&[u8]> {
        self.chunks
            .get(&(local_x, local_z))
            .and_then(|e| e.data.as_deref())
    }

    /// Create a new empty region.
    pub fn new(x: i32, z: i32) -> Self {
        Self {
            x,
            z,
            chunks: HashMap::new(),
        }
    }

    /// Set chunk data at local coordinates (0..32, 0..32).
    /// `data` should be decompressed NBT bytes.
    pub fn set_chunk_data(&mut self, local_x: u8, local_z: u8, data: Vec<u8>) {
        self.chunks.insert((local_x, local_z), ChunkEntry {
            offset: 0,
            size: 0,
            timestamp: 0,
            data: Some(data),
        });
    }

    /// List all chunk coordinates present in this region.
    pub fn chunk_coords(&self) -> Vec<(u8, u8)> {
        let mut coords: Vec<_> = self.chunks.keys().copied().collect();
        coords.sort();
        coords
    }

    /// Number of chunks in this region.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Serialize region back to .mca bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let sector_size: u64 = 4096;
        let mut locations = [0u32; 1024];
        let mut timestamps = [0u32; 1024];
        let mut sector_data: Vec<Vec<u8>> = Vec::new();

        for i in 0..1024 {
            let local_x = (i % 32) as u8;
            let local_z = (i / 32) as u8;

            if let Some(entry) = self.chunks.get(&(local_x, local_z)) {
                timestamps[i] = entry.timestamp;

                // Serialize NBT and compress with Zlib
                let compressed = compression::compress(
                    entry.data.as_deref().unwrap_or_default(),
                    CompressionType::Zlib,
                )?;

                // Prepend: length (4 bytes BE) + compression type (1 byte)
                let total_len = 1 + compressed.len();
                let mut sector = Vec::with_capacity(4 + total_len);
                sector.extend(&(total_len as u32).to_be_bytes());
                sector.push(CompressionType::Zlib.id());
                sector.extend(&compressed);

                // Pad to sector boundary
                while sector.len() % sector_size as usize != 0 {
                    sector.push(0);
                }

                let offset = 2 + sector_data.len() as u32;
                locations[i] =
                    (offset << 8) | ((sector.len() / sector_size as usize) as u32 & 0xFF);
                sector_data.push(sector);
            }
        }

        // Build output: header + sector data
        let mut output =
            Vec::with_capacity(2 * 4096 + sector_data.iter().map(|s| s.len()).sum::<usize>());

        // Location table
        for loc in locations.iter() {
            output.extend(&loc.to_be_bytes());
        }
        // Timestamp table
        for ts in timestamps.iter() {
            output.extend(&ts.to_be_bytes());
        }

        // Pad header to exactly 2 sectors
        while output.len() < 2 * sector_size as usize {
            output.push(0);
        }

        // Sector data
        for sector in &sector_data {
            output.extend(sector);
        }

        Ok(output)
    }
}
