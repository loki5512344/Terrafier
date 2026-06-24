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
