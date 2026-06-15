//! World model — the core data structures.
//!
//! Contains World, Dimension, Tile, Terrain types and related utilities.

pub mod brush;
pub mod dimension;
pub mod layers;
pub mod platform;
pub mod terrain;
pub mod tile;
pub mod world;

pub use dimension::Dimension;
pub use platform::Platform;
pub use terrain::Terrain;
pub use tile::Tile;
pub use world::World;
