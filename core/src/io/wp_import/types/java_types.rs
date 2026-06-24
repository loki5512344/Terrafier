use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WpImportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid WorldPainter file: {0}")]
    InvalidFormat(String),
    #[error("Unsupported WorldPainter version: {0}")]
    UnsupportedVersion(i32),
    #[error("Compression error: {0}")]
    Compression(String),
    #[error("Missing data: {0}")]
    MissingData(String),
}

pub type Result<T> = std::result::Result<T, WpImportError>;

pub const TC_NULL: u8 = 0x70;
pub const TC_REFERENCE: u8 = 0x71;
pub const TC_CLASSDESC: u8 = 0x72;
pub const TC_OBJECT: u8 = 0x73;
pub const TC_STRING: u8 = 0x74;
pub const TC_ARRAY: u8 = 0x75;
pub const TC_CLASS: u8 = 0x76;
pub const TC_BLOCKDATA: u8 = 0x77;
pub const TC_ENDBLOCKDATA: u8 = 0x78;
pub const TC_RESET: u8 = 0x79;
pub const TC_BLOCKDATALONG: u8 = 0x7A;
pub const TC_LONGSTRING: u8 = 0x7C;
pub const TC_PROXYCLASSDESC: u8 = 0x7D;
pub const TC_ENUM: u8 = 0x7E;
pub const SC_WRITE_METHOD: u8 = 0x01;
pub const BASE_WIRE_HANDLE: u32 = 0x7E0000;
pub const PRIM_BYTE: u8 = b'B';
pub const PRIM_CHAR: u8 = b'C';
pub const PRIM_DOUBLE: u8 = b'D';
pub const PRIM_FLOAT: u8 = b'F';
pub const PRIM_INT: u8 = b'I';
pub const PRIM_LONG: u8 = b'J';
pub const PRIM_SHORT: u8 = b'S';
pub const PRIM_BOOL: u8 = b'Z';

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub type_code: u8,
}

#[derive(Debug, Clone)]
pub struct ClassDesc {
    pub name: String,
    pub flags: u8,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Clone)]
pub enum JvmValue {
    Null,
    String(String),
    ClassDesc(ClassDesc),
    Object(HashMap<String, JvmValue>),
    ByteArray(Vec<i8>),
    ShortArray(Vec<i16>),
    IntArray(Vec<i32>),
    LongArray(()),
    Int(i32),
    Long(i64),
    Skipped,
}

impl JvmValue {
    pub fn as_string(&self) -> Option<String> {
        match self {
            JvmValue::String(s) => Some(s.clone()),
            JvmValue::Object(map) => {
                for key in &["name", "key", "id", "value"] {
                    if let Some(val) = map.get(*key)
                        && let JvmValue::String(s) = val
                    {
                        return Some(s.clone());
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            JvmValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            JvmValue::Long(v) => Some(*v as u64),
            JvmValue::Int(v) => Some(*v as u64),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JvmValue>> {
        match self {
            JvmValue::Object(m) => Some(m),
            _ => None,
        }
    }
}
