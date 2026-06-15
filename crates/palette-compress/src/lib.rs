//! Block palette compression for Minecraft Anvil format.
//!
//! Minecraft 1.13+ uses a palette-based block storage format where each
//! section stores a list of unique block states (palette) and indices
//! into that palette (packed into a BitArray). This crate provides
//! the building blocks for reading and writing compressed block data.

pub mod bits;
pub mod palette;
pub mod section;

pub use bits::BitArray;
pub use palette::{BlockPalette, BlockState};
pub use section::SectionData;
