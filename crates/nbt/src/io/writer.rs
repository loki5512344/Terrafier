//! NBT binary writer for Java Edition (Big Endian).

use std::io::Write;
use thiserror::Error;

use crate::tag::Tag;

#[derive(Error, Debug)]
pub enum WriteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unsupported tag type in list: {0}")]
    UnsupportedListType(u8),
    #[error("Empty list cannot determine element type")]
    EmptyList,
}

pub type Result<T> = std::result::Result<T, WriteError>;

/// Serialize a Tag tree to bytes (Big Endian, no compression).
pub fn to_bytes(tag: &Tag) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut w = NbtWriter::new(&mut buf);
    w.write_tag_compound_root(tag)?;
    Ok(buf)
}

/// Serialize a Tag tree to gzip-compressed bytes.
pub fn to_gzip_bytes(tag: &Tag) -> Result<Vec<u8>> {
    let raw = to_bytes(tag)?;
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&raw)?;
    Ok(encoder.finish()?)
}

struct NbtWriter<W: Write> {
    inner: W,
}

impl<W: Write> NbtWriter<W> {
    fn new(inner: W) -> Self {
        Self { inner }
    }

    fn write_u8(&mut self, val: u8) -> Result<()> {
        self.inner.write_all(&[val])?;
        Ok(())
    }

    fn write_i16_be(&mut self, val: i16) -> Result<()> {
        self.inner.write_all(&val.to_be_bytes())?;
        Ok(())
    }

    fn write_i32_be(&mut self, val: i32) -> Result<()> {
        self.inner.write_all(&val.to_be_bytes())?;
        Ok(())
    }

    fn write_i64_be(&mut self, val: i64) -> Result<()> {
        self.inner.write_all(&val.to_be_bytes())?;
        Ok(())
    }

    fn write_f32_be(&mut self, val: f32) -> Result<()> {
        self.inner.write_all(&val.to_be_bytes())?;
        Ok(())
    }

    fn write_f64_be(&mut self, val: f64) -> Result<()> {
        self.inner.write_all(&val.to_be_bytes())?;
        Ok(())
    }

    fn write_string(&mut self, s: &str) -> Result<()> {
        let bytes = s.as_bytes();
        if bytes.len() > u16::MAX as usize {
            return Err(WriteError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "String too long for NBT",
            )));
        }
        self.write_i16_be(bytes.len() as i16)?;
        self.inner.write_all(bytes)?;
        Ok(())
    }

    fn write_tag(&mut self, tag: &Tag, name: Option<&str>) -> Result<()> {
        self.write_u8(tag.id())?;
        if let Some(n) = name {
            self.write_string(n)?;
        }
        self.write_tag_payload(tag)?;
        Ok(())
    }

    fn write_tag_payload(&mut self, tag: &Tag) -> Result<()> {
        match tag {
            Tag::End => {}
            Tag::Byte(v) => self.write_u8(*v as u8)?,
            Tag::Short(v) => self.write_i16_be(*v)?,
            Tag::Int(v) => self.write_i32_be(*v)?,
            Tag::Long(v) => self.write_i64_be(*v)?,
            Tag::Float(v) => self.write_f32_be(*v)?,
            Tag::Double(v) => self.write_f64_be(*v)?,
            Tag::String(v) => self.write_string(v)?,
            Tag::List(items) => {
                if items.is_empty() {
                    self.write_u8(1)?; // TAG_Byte as fallback
                    self.write_i32_be(0)?;
                } else {
                    let elem_type = items[0].id();
                    self.write_u8(elem_type)?;
                    self.write_i32_be(items.len() as i32)?;
                    for item in items {
                        self.write_tag_payload(item)?;
                    }
                }
            }
            Tag::Compound(map) => {
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                for key in keys {
                    if let Some(val) = map.get(key) {
                        self.write_tag(val, Some(key))?;
                    }
                }
                self.write_u8(0)?; // TAG_End
            }
            Tag::ByteArray(v) => {
                self.write_i32_be(v.len() as i32)?;
                for b in v {
                    self.inner.write_all(&[*b as u8])?;
                }
            }
            Tag::IntArray(v) => {
                self.write_i32_be(v.len() as i32)?;
                for n in v {
                    self.write_i32_be(*n)?;
                }
            }
            Tag::LongArray(v) => {
                self.write_i32_be(v.len() as i32)?;
                for n in v {
                    self.write_i64_be(*n)?;
                }
            }
        }
        Ok(())
    }

    fn write_tag_compound_root(&mut self, tag: &Tag) -> Result<()> {
        match tag {
            Tag::Compound(map) => {
                self.write_u8(9)?; // TAG_Compound
                self.write_string("")?; // empty root name
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                for key in keys {
                    if let Some(val) = map.get(key) {
                        self.write_tag(val, Some(key))?;
                    }
                }
                self.write_u8(0)?; // TAG_End
                Ok(())
            }
            _ => Err(WriteError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Root tag must be Compound",
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::reader;
    use std::collections::HashMap;

    #[test]
    fn test_roundtrip_byte() {
        let tag = Tag::Compound(HashMap::from([("val".into(), Tag::Byte(42))]));
        let bytes = to_bytes(&tag).unwrap();
        let parsed = reader::read_bytes(&bytes).unwrap();
        assert_eq!(tag, parsed);
    }

    #[test]
    fn test_roundtrip_nested() {
        let inner = Tag::Compound(HashMap::from([
            ("x".into(), Tag::Int(100)),
            ("y".into(), Tag::Int(200)),
        ]));
        let tag = Tag::Compound(HashMap::from([("pos".into(), inner)]));
        let bytes = to_bytes(&tag).unwrap();
        let parsed = reader::read_bytes(&bytes).unwrap();
        assert_eq!(tag, parsed);
    }

    #[test]
    fn test_roundtrip_all_types() {
        let mut map = HashMap::new();
        map.insert("byte".into(), Tag::Byte(1));
        map.insert("short".into(), Tag::Short(2));
        map.insert("int".into(), Tag::Int(3));
        map.insert("long".into(), Tag::Long(4));
        map.insert("float".into(), Tag::Float(5.0));
        map.insert("double".into(), Tag::Double(6.0));
        map.insert("string".into(), Tag::String("hello".into()));
        map.insert("bytearray".into(), Tag::ByteArray(vec![1, 2, 3]));
        map.insert("intarray".into(), Tag::IntArray(vec![4, 5, 6]));
        map.insert("longarray".into(), Tag::LongArray(vec![7, 8, 9]));
        let tag = Tag::Compound(map);
        let bytes = to_bytes(&tag).unwrap();
        let parsed = reader::read_bytes(&bytes).unwrap();
        assert_eq!(tag, parsed);
    }

    #[test]
    fn test_gzip_roundtrip() {
        let tag = Tag::Compound(HashMap::from([("val".into(), Tag::Int(12345))]));
        let gz = to_gzip_bytes(&tag).unwrap();
        let parsed = reader::read_gzip(&gz).unwrap();
        assert_eq!(tag, parsed);
    }
}
