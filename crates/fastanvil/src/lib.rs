//! # Terrafier FastAnvil
//!
//! Fast reading and writing of Minecraft Anvil (.mca) and MCRegion (.mcr) files.
//!
//! Supports:
//! - Reading existing regions
//! - Writing new regions
//! - Chunk-level access (compressed NBT data)
//! - Parallel chunk processing

pub mod compression;
pub mod io;

pub use io::chunk;
pub use io::region;
