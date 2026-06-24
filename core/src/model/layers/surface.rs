use super::{DataSize, Layer};

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
