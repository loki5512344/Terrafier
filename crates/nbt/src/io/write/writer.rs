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

pub(crate) struct NbtWriter<W: Write> {
    pub(crate) inner: W,
}

impl<W: Write> NbtWriter<W> {
    pub(crate) fn new(inner: W) -> Self {
        Self { inner }
    }

    pub(crate) fn write_u8(&mut self, val: u8) -> Result<()> {
        self.inner.write_all(&[val])?;
        Ok(())
    }

    pub(crate) fn write_i16_be(&mut self, val: i16) -> Result<()> {
        self.inner.write_all(&val.to_be_bytes())?;
        Ok(())
    }

    pub(crate) fn write_i32_be(&mut self, val: i32) -> Result<()> {
        self.inner.write_all(&val.to_be_bytes())?;
        Ok(())
    }

    pub(crate) fn write_i64_be(&mut self, val: i64) -> Result<()> {
        self.inner.write_all(&val.to_be_bytes())?;
        Ok(())
    }

    pub(crate) fn write_f32_be(&mut self, val: f32) -> Result<()> {
        self.inner.write_all(&val.to_be_bytes())?;
        Ok(())
    }

    pub(crate) fn write_f64_be(&mut self, val: f64) -> Result<()> {
        self.inner.write_all(&val.to_be_bytes())?;
        Ok(())
    }

    pub(crate) fn write_string(&mut self, s: &str) -> Result<()> {
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
}
