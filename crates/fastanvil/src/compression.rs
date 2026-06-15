//! Compression utilities for Minecraft region files.
//!
//! Minecraft uses Zlib (deflate) compression for chunk data in .mca files.
//! MCRegion files use GZip compression.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Decompression error: {0}")]
    Decompress(String),
    #[error("Compression error: {0}")]
    Compress(String),
    #[error("Unknown compression scheme: {0}")]
    UnknownScheme(u8),
}

pub type Result<T> = std::result::Result<T, CompressionError>;

/// Compression type identifiers used in .mca chunk headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    /// GZip compression (MCRegion, type=1)
    GZip,
    /// Zlib deflate compression (Anvil / .mca, type=2)
    Zlib,
    /// Uncompressed (type=3)
    Uncompressed,
}

impl CompressionType {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            1 => Some(CompressionType::GZip),
            2 => Some(CompressionType::Zlib),
            3 => Some(CompressionType::Uncompressed),
            _ => None,
        }
    }

    pub fn id(&self) -> u8 {
        match self {
            CompressionType::GZip => 1,
            CompressionType::Zlib => 2,
            CompressionType::Uncompressed => 3,
        }
    }
}

/// Decompress chunk data given the compression type.
pub fn decompress(data: &[u8], scheme: CompressionType) -> Result<Vec<u8>> {
    use std::io::Read;
    match scheme {
        CompressionType::GZip => {
            let mut dec = flate2::read::GzDecoder::new(data);
            let mut buf = Vec::new();
            dec.read_to_end(&mut buf)
                .map_err(|e| CompressionError::Decompress(e.to_string()))?;
            Ok(buf)
        }
        CompressionType::Zlib => {
            let mut dec = flate2::read::ZlibDecoder::new(data);
            let mut buf = Vec::new();
            dec.read_to_end(&mut buf)
                .map_err(|e| CompressionError::Decompress(e.to_string()))?;
            Ok(buf)
        }
        CompressionType::Uncompressed => Ok(data.to_vec()),
    }
}

/// Compress chunk data using the specified compression scheme.
pub fn compress(data: &[u8], scheme: CompressionType) -> Result<Vec<u8>> {
    use std::io::Write;
    match scheme {
        CompressionType::GZip => {
            let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            enc.write_all(data)
                .map_err(|e| CompressionError::Compress(e.to_string()))?;
            enc.finish()
                .map_err(|e| CompressionError::Compress(e.to_string()))
        }
        CompressionType::Zlib => {
            let mut enc =
                flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            enc.write_all(data)
                .map_err(|e| CompressionError::Compress(e.to_string()))?;
            enc.finish()
                .map_err(|e| CompressionError::Compress(e.to_string()))
        }
        CompressionType::Uncompressed => Ok(data.to_vec()),
    }
}
