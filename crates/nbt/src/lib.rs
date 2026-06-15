//! Zero-copy NBT (Named Binary Tag) parser and writer.
//!
//! Implements the Minecraft NBT format with optional serde support.
//! Uses zero-copy deserialization where possible to minimize allocations.
//!
//! ## Format support
//!
//! - All tag types: Byte, Short, Int, Long, Float, Double, String,
//!   List, Compound, IntArray, LongArray, ByteArray
//! - GZip compressed streams
//! - Java edition (Big Endian) and Bedrock edition (Little Endian) variants

#![deny(unsafe_code)]

pub mod io;
pub mod tag;

pub use io::reader;
pub use io::writer;
pub use tag::Tag;
