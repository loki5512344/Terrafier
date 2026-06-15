//! Biome database for Terrafier.
//!
//! Maps biome IDs to names, colours, and patterns for all supported
//! Minecraft versions (1.0 through 1.21+).

pub mod colour;
pub mod db;

pub use colour::BiomeColour;
pub use db::{BiomeDb, BiomeEntry};
