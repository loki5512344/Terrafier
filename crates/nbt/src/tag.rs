//! NBT tag types.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    List(Vec<Tag>),
    Compound(HashMap<String, Tag>),
    ByteArray(Vec<i8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Tag {
    pub fn name(&self) -> &'static str {
        match self {
            Tag::End => "TAG_End",
            Tag::Byte(_) => "TAG_Byte",
            Tag::Short(_) => "TAG_Short",
            Tag::Int(_) => "TAG_Int",
            Tag::Long(_) => "TAG_Long",
            Tag::Float(_) => "TAG_Float",
            Tag::Double(_) => "TAG_Double",
            Tag::String(_) => "TAG_String",
            Tag::List(_) => "TAG_List",
            Tag::Compound(_) => "TAG_Compound",
            Tag::ByteArray(_) => "TAG_Byte_Array",
            Tag::IntArray(_) => "TAG_Int_Array",
            Tag::LongArray(_) => "TAG_Long_Array",
        }
    }

    pub fn id(&self) -> u8 {
        match self {
            Tag::End => 0,
            Tag::Byte(_) => 1,
            Tag::Short(_) => 2,
            Tag::Int(_) => 3,
            Tag::Long(_) => 4,
            Tag::Float(_) => 5,
            Tag::Double(_) => 6,
            Tag::String(_) => 7,
            Tag::List(_) => 8,
            Tag::Compound(_) => 9,
            Tag::ByteArray(_) => 10,
            Tag::IntArray(_) => 11,
            Tag::LongArray(_) => 12,
        }
    }
}
