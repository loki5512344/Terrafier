//! Layer system — defines the Layer trait and built-in layer types.

pub mod caves;
pub mod surface;

pub use caves::{CavesLayer, FrostLayer, RiverLayer};
pub use surface::{BiomeLayer, ResourcesLayer, TreesLayer};

pub trait Layer: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn data_size(&self) -> DataSize;
    fn priority(&self) -> i32;
}

pub enum DataSize {
    Bit,
    Nibble,
    Byte,
    Int,
}

pub const LAYER_CAVES: u32 = 1;
pub const LAYER_RIVER: u32 = 2;
pub const LAYER_FROST: u32 = 3;
pub const LAYER_TREES: u32 = 4;
pub const LAYER_BIOME: u32 = 5;
pub const LAYER_RESOURCES: u32 = 6;
