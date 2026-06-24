//! World model — the core data structures.
//!
//! Contains World, Dimension, Tile, Terrain types and related utilities.

pub mod layers;
pub mod tile;
pub mod types;
pub mod world;

pub use tile::Tile;
pub use types::{Brush, Platform, SymmetricBrush, Terrain};
pub use world::*;
