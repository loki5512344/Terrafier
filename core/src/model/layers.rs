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
