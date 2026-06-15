use super::{DataSize, Layer};

pub struct CavesLayer;
impl Layer for CavesLayer {
    fn id(&self) -> &'static str {
        "caves"
    }
    fn name(&self) -> &'static str {
        "Caves"
    }
    fn data_size(&self) -> DataSize {
        DataSize::Nibble
    }
    fn priority(&self) -> i32 {
        10
    }
}

pub struct RiverLayer;
impl Layer for RiverLayer {
    fn id(&self) -> &'static str {
        "river"
    }
    fn name(&self) -> &'static str {
        "River"
    }
    fn data_size(&self) -> DataSize {
        DataSize::Byte
    }
    fn priority(&self) -> i32 {
        20
    }
}

pub struct FrostLayer;
impl Layer for FrostLayer {
    fn id(&self) -> &'static str {
        "frost"
    }
    fn name(&self) -> &'static str {
        "Frost"
    }
    fn data_size(&self) -> DataSize {
        DataSize::Bit
    }
    fn priority(&self) -> i32 {
        30
    }
}

pub struct TreesLayer;
impl Layer for TreesLayer {
    fn id(&self) -> &'static str {
        "trees"
    }
    fn name(&self) -> &'static str {
        "Trees"
    }
    fn data_size(&self) -> DataSize {
        DataSize::Byte
    }
    fn priority(&self) -> i32 {
        40
    }
}

pub struct BiomeLayer;
impl Layer for BiomeLayer {
    fn id(&self) -> &'static str {
        "biome"
    }
    fn name(&self) -> &'static str {
        "Biome"
    }
    fn data_size(&self) -> DataSize {
        DataSize::Byte
    }
    fn priority(&self) -> i32 {
        50
    }
}

pub struct ResourcesLayer;
impl Layer for ResourcesLayer {
    fn id(&self) -> &'static str {
        "resources"
    }
    fn name(&self) -> &'static str {
        "Resources"
    }
    fn data_size(&self) -> DataSize {
        DataSize::Nibble
    }
    fn priority(&self) -> i32 {
        60
    }
}
