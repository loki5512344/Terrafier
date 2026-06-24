//! NBT binary reader for Java Edition (Big Endian).

use std::io::{Cursor, Read};
use thiserror::Error;

use crate::tag::Tag;

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unknown tag type: {0}")]
    UnknownTagType(u8),
    #[error("Invalid string length: {0}")]
    InvalidStringLength(usize),
    #[error("Invalid array length: {0}")]
    InvalidArrayLength(usize),
}

pub type Result<T> = std::result::Result<T, ReadError>;

pub fn read_bytes(data: &[u8]) -> Result<Tag> {
    let cursor = Cursor::new(data);
    let mut de = NbtReader::new(cursor);
    de.read_tag_compound_root()
}

pub fn read_gzip(data: &[u8]) -> Result<Tag> {
    let mut dec = flate2::read::GzDecoder::new(data);
    let mut buf = Vec::new();
    dec.read_to_end(&mut buf)?;
    read_bytes(&buf)
}

pub(crate) struct NbtReader<R: Read> {
    pub(crate) inner: R,
    pub(crate) buf: Vec<u8>,
}

impl<R: Read> NbtReader<R> {
    pub(crate) fn new(inner: R) -> Self {
        Self {
            inner,
            buf: Vec::new(),
        }
    }

    pub(crate) fn read_exact(&mut self, len: usize) -> Result<&[u8]> {
        self.buf.clear();
        self.buf.resize(len, 0);
        self.inner.read_exact(&mut self.buf)?;
        Ok(&self.buf)
    }

    pub(crate) fn read_u8(&mut self) -> Result<u8> {
        let mut byte = [0u8; 1];
        self.inner.read_exact(&mut byte)?;
        Ok(byte[0])
    }

    pub(crate) fn read_i16_be(&mut self) -> Result<i16> {
        let b = self.read_exact(2)?;
        Ok(i16::from_be_bytes([b[0], b[1]]))
    }

    pub(crate) fn read_i32_be(&mut self) -> Result<i32> {
        let b = self.read_exact(4)?;
        Ok(i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub(crate) fn read_i64_be(&mut self) -> Result<i64> {
        let b = self.read_exact(8)?;
        Ok(i64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    pub(crate) fn read_f32_be(&mut self) -> Result<f32> {
        let b = self.read_exact(4)?;
        Ok(f32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub(crate) fn read_f64_be(&mut self) -> Result<f64> {
        let b = self.read_exact(8)?;
        Ok(f64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    pub(crate) fn read_string(&mut self) -> Result<String> {
        let len = self.read_i16_be()? as u16 as usize;
        let bytes = self.read_exact(len)?.to_vec();
        String::from_utf8(bytes).map_err(|_| ReadError::InvalidStringLength(len))
    }
}
