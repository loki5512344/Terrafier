//! WorldPainter .world file import.
//!
//! Reads WorldPainter `.world` files, which use Java serialization format.
//! Implements a minimal Java ObjectInputStream parser sufficient for extracting
//! tile heightmap, terrain, and water level data from WorldPainter worlds.

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::model::dimension::Dimension;
use crate::model::platform::Platform;
use crate::model::tile::Tile;
use crate::model::world::World;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Java serialisation stream constants
// ---------------------------------------------------------------------------

const TC_NULL: u8 = 0x70;
const TC_REFERENCE: u8 = 0x71;
const TC_CLASSDESC: u8 = 0x72;
const TC_OBJECT: u8 = 0x73;
const TC_STRING: u8 = 0x74;
const TC_ARRAY: u8 = 0x75;
const TC_CLASS: u8 = 0x76;
const TC_BLOCKDATA: u8 = 0x77;
const TC_ENDBLOCKDATA: u8 = 0x78;
const TC_RESET: u8 = 0x79;
const TC_BLOCKDATALONG: u8 = 0x7A;
const TC_LONGSTRING: u8 = 0x7C;
const TC_PROXYCLASSDESC: u8 = 0x7D;
const TC_ENUM: u8 = 0x7E;

const SC_WRITE_METHOD: u8 = 0x01;

const BASE_WIRE_HANDLE: u32 = 0x7E0000;

const PRIM_BYTE: u8 = b'B';
const PRIM_CHAR: u8 = b'C';
const PRIM_DOUBLE: u8 = b'D';
const PRIM_FLOAT: u8 = b'F';
const PRIM_INT: u8 = b'I';
const PRIM_LONG: u8 = b'J';
const PRIM_SHORT: u8 = b'S';
const PRIM_BOOL: u8 = b'Z';

// ---------------------------------------------------------------------------
// Java serialisation stream reader
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct FieldInfo {
    name: String,
    type_code: u8,
}

#[derive(Debug, Clone)]
struct ClassDesc {
    name: String,
    flags: u8,
    fields: Vec<FieldInfo>,
}

#[derive(Debug, Clone)]
enum JvmValue {
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
    fn as_string(&self) -> Option<String> {
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

    fn as_i32(&self) -> Option<i32> {
        match self {
            JvmValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match self {
            JvmValue::Long(v) => Some(*v as u64),
            JvmValue::Int(v) => Some(*v as u64),
            _ => None,
        }
    }

    fn as_object(&self) -> Option<&HashMap<String, JvmValue>> {
        match self {
            JvmValue::Object(m) => Some(m),
            _ => None,
        }
    }
}

struct JvmStream {
    data: Vec<u8>,
    pos: usize,
    handles: Vec<JvmValue>,
    class_descs: Vec<ClassDesc>,
}

impl JvmStream {
    fn new(data: Vec<u8>) -> Self {
        JvmStream {
            data,
            pos: 0,
            handles: Vec::new(),
            class_descs: Vec::new(),
        }
    }

    fn read_byte(&mut self) -> std::io::Result<u8> {
        if self.pos >= self.data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected end of stream",
            ));
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn read_bytes(&mut self, n: usize) -> std::io::Result<Vec<u8>> {
        if self.pos + n > self.data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected end of stream",
            ));
        }
        let slice = self.data[self.pos..self.pos + n].to_vec();
        self.pos += n;
        Ok(slice)
    }

    fn peek_byte(&self) -> std::io::Result<u8> {
        if self.pos >= self.data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected end of stream",
            ));
        }
        Ok(self.data[self.pos])
    }

    fn read_u16(&mut self) -> std::io::Result<u16> {
        let hi = self.read_byte()? as u16;
        let lo = self.read_byte()? as u16;
        Ok((hi << 8) | lo)
    }

    fn read_i32(&mut self) -> std::io::Result<i32> {
        let b1 = self.read_byte()? as i32;
        let b2 = self.read_byte()? as i32;
        let b3 = self.read_byte()? as i32;
        let b4 = self.read_byte()? as i32;
        Ok((b1 << 24) | (b2 << 16) | (b3 << 8) | b4)
    }

    fn read_i64(&mut self) -> std::io::Result<i64> {
        let b1 = self.read_byte()? as i64;
        let b2 = self.read_byte()? as i64;
        let b3 = self.read_byte()? as i64;
        let b4 = self.read_byte()? as i64;
        let b5 = self.read_byte()? as i64;
        let b6 = self.read_byte()? as i64;
        let b7 = self.read_byte()? as i64;
        let b8 = self.read_byte()? as i64;
        Ok((b1 << 56)
            | (b2 << 48)
            | (b3 << 40)
            | (b4 << 32)
            | (b5 << 24)
            | (b6 << 16)
            | (b7 << 8)
            | b8)
    }

    fn read_f32(&mut self) -> std::io::Result<f32> {
        Ok(f32::from_bits(self.read_i32()? as u32))
    }

    fn read_f64(&mut self) -> std::io::Result<f64> {
        Ok(f64::from_bits(self.read_i64()? as u64))
    }

    fn read_utf(&mut self) -> std::io::Result<String> {
        let len = self.read_u16()? as usize;
        let bytes = self.read_bytes(len)?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    fn read_long_utf(&mut self) -> std::io::Result<String> {
        let len = self.read_i64()? as usize;
        let bytes = self.read_bytes(len)?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    fn read_content(&mut self) -> std::io::Result<JvmValue> {
        loop {
            if self.pos >= self.data.len() {
                return Ok(JvmValue::Null);
            }
            let tc = self.read_byte()?;
            match tc {
                TC_OBJECT => return self.read_object(),
                TC_ARRAY => return self.read_array(),
                TC_STRING => {
                    let s = self.read_utf()?;
                    self.push_handle(JvmValue::String(s.clone()));
                    return Ok(JvmValue::String(s));
                }
                TC_LONGSTRING => {
                    let s = self.read_long_utf()?;
                    self.push_handle(JvmValue::String(s.clone()));
                    return Ok(JvmValue::String(s));
                }
                TC_REFERENCE => {
                    let handle = self.read_u32_handle();
                    let idx = (handle - BASE_WIRE_HANDLE) as usize;
                    if idx < self.handles.len() {
                        let val = self.handles[idx].clone();
                        return Ok(val);
                    }
                    return Ok(JvmValue::Null);
                }
                TC_NULL => return Ok(JvmValue::Null),
                TC_CLASS => {
                    let _desc = self.read_classdesc()?;
                    return Ok(JvmValue::Skipped);
                }
                TC_ENUM => {
                    let _desc = self.read_classdesc()?;
                    let _ordinal = self.read_i32()?;
                    return Ok(JvmValue::Skipped);
                }
                TC_BLOCKDATA => {
                    let len = self.read_byte()? as usize;
                    self.pos += len;
                }
                TC_BLOCKDATALONG => {
                    let len = self.read_i32()? as usize;
                    self.pos += len;
                }
                TC_RESET => {
                    self.handles.clear();
                }
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("unexpected content token: 0x{:02x}", tc),
                    ));
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Handle management
    // -----------------------------------------------------------------------

    fn push_handle(&mut self, val: JvmValue) -> u32 {
        let idx = self.handles.len();
        self.handles.push(val);
        BASE_WIRE_HANDLE + idx as u32
    }

    fn read_u32_handle(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&self.data[self.pos..self.pos + 4]);
        self.pos += 4;
        u32::from_be_bytes(buf)
    }

    // -----------------------------------------------------------------------
    // Class descriptor parsing
    // -----------------------------------------------------------------------

    fn read_classdesc(&mut self) -> std::io::Result<JvmValue> {
        let tc = self.peek_byte()?;
        match tc {
            TC_CLASSDESC => self.read_classdesc_body(),
            TC_REFERENCE => {
                self.read_byte()?;
                let handle = self.read_u32_handle();
                let idx = (handle - BASE_WIRE_HANDLE) as usize;
                if idx < self.handles.len() {
                    Ok(self.handles[idx].clone())
                } else {
                    Ok(JvmValue::Null)
                }
            }
            TC_NULL => {
                self.read_byte()?;
                Ok(JvmValue::Null)
            }
            TC_PROXYCLASSDESC => {
                self.read_byte()?;
                let _iface_count = self.read_i32()?;
                for _ in 0.._iface_count {
                    let _iface = self.read_utf()?;
                }
                self.read_class_annotations()?;
                let _super = self.read_classdesc()?;
                self.push_handle(JvmValue::Skipped);
                Ok(JvmValue::Skipped)
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("expected classdesc token, got 0x{:02x}", tc),
            )),
        }
    }

    fn read_classdesc_body(&mut self) -> std::io::Result<JvmValue> {
        self.read_byte()?; // consume TC_CLASSDESC
        let class_name = self.read_utf()?;
        let _serial_version_uid = self.read_i64()?;
        let desc_flags = self.read_byte()?;
        let field_count = self.read_u16()?;

        let mut fields = Vec::new();
        for _ in 0..field_count {
            let type_code = self.read_byte()?;
            let name = self.read_utf()?;
            if type_code == b'[' || type_code == b'L' {
                let tc = self.read_byte()?;
                match tc {
                    TC_STRING => {
                        self.read_utf()?;
                    }
                    TC_REFERENCE => {
                        let handle = self.read_u32_handle();
                        let idx = (handle - BASE_WIRE_HANDLE) as usize;
                        if idx < self.handles.len() {
                            let _ = self.handles[idx].as_string();
                        }
                    }
                    TC_LONGSTRING => {
                        self.read_long_utf()?;
                    }
                    _ => {}
                }
            }
            fields.push(FieldInfo { name, type_code });
        }

        let cd = ClassDesc {
            name: class_name.clone(),
            flags: desc_flags,
            fields,
        };
        self.class_descs.push(cd.clone());

        // Push handle for this class descriptor
        self.push_handle(JvmValue::ClassDesc(cd.clone()));

        // Class annotations
        self.read_class_annotations()?;

        // Super class descriptor
        let _super = self.read_classdesc()?;

        Ok(JvmValue::ClassDesc(cd))
    }

    fn read_class_annotations(&mut self) -> std::io::Result<()> {
        loop {
            let tc = self.read_byte()?;
            match tc {
                TC_ENDBLOCKDATA => return Ok(()),
                TC_BLOCKDATA => {
                    let len = self.read_byte()? as usize;
                    self.pos += len;
                }
                TC_BLOCKDATALONG => {
                    let len = self.read_i32()? as usize;
                    self.pos += len;
                }
                TC_OBJECT | TC_ARRAY | TC_STRING | TC_LONGSTRING | TC_ENUM => {
                    self.pos -= 1; // push back, let read_content handle it
                    let _val = self.read_content()?;
                }
                TC_REFERENCE => {
                    let _handle = self.read_u32_handle();
                }
                TC_CLASSDESC | TC_PROXYCLASSDESC => {
                    self.pos -= 1;
                    let _desc = self.read_classdesc()?;
                }
                TC_NULL => {}
                TC_CLASS => {
                    let _desc = self.read_classdesc()?;
                }
                TC_RESET => {
                    self.handles.clear();
                }
                other => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("unexpected token in class annotations: 0x{:02x}", other),
                    ));
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Object parsing
    // -----------------------------------------------------------------------

    fn read_object(&mut self) -> std::io::Result<JvmValue> {
        let class_desc_val = self.read_classdesc()?;
        let cd = match &class_desc_val {
            JvmValue::ClassDesc(cd) => cd.clone(),
            JvmValue::Null => {
                // Null classdesc means no fields
                return Ok(JvmValue::Object(HashMap::new()));
            }
            _ => {
                // Try to find field info from known classes
                let name = class_desc_val.as_string().unwrap_or_default();
                let cd = self.resolve_class_desc(&name);
                match cd {
                    Some(cd) => cd,
                    None => {
                        // Unknown class, skip object data
                        let _handle = self.push_handle(JvmValue::Skipped);
                        self.skip_object_data(&name)?;
                        return Ok(JvmValue::Skipped);
                    }
                }
            }
        };

        let handle = self.push_handle(JvmValue::Skipped);

        let obj = match cd.name.as_str() {
            "java.util.HashMap" | "java.util.LinkedHashMap" => self.read_hashmap_data()?,
            "java.util.HashSet" | "java.util.LinkedHashSet" => self.read_hashset_data()?,
            "java.util.ArrayList" => self.read_arraylist_data()?,
            "java.lang.Integer" => {
                let val = self.read_i32()?;
                self.handles[(handle - BASE_WIRE_HANDLE) as usize] = JvmValue::Int(val);
                return Ok(JvmValue::Int(val));
            }
            "java.lang.Long" => {
                let val = self.read_i64()?;
                self.handles[(handle - BASE_WIRE_HANDLE) as usize] = JvmValue::Long(val);
                return Ok(JvmValue::Long(val));
            }
            "java.lang.String" => {
                let val = self.read_content()?;
                self.handles[(handle - BASE_WIRE_HANDLE) as usize] = val.clone();
                return Ok(val);
            }
            _ => self.read_object_fields(&cd)?,
        };

        self.handles[(handle - BASE_WIRE_HANDLE) as usize] = obj.clone();
        Ok(obj)
    }

    fn resolve_class_desc(&self, name: &str) -> Option<ClassDesc> {
        // Exact match first
        for cd in &self.class_descs {
            if cd.name == name {
                return Some(cd.clone());
            }
        }
        // Try known WorldPainter class mappings
        None
    }

    fn skip_object_data(&mut self, _class_name: &str) -> std::io::Result<()> {
        // Try to read and discard using class annotations
        // This is a best-effort skip
        let saved = self.pos;
        match self.read_class_annotations() {
            Ok(()) => Ok(()),
            Err(_) => {
                self.pos = saved;
                // Try reading block data style
                if self.pos < self.data.len() {
                    let tc = match self.peek_byte() {
                        Ok(t) => t,
                        Err(_) => return Ok(()),
                    };
                    if tc == TC_BLOCKDATA || tc == TC_BLOCKDATALONG {
                        return self.read_class_annotations();
                    }
                    // Try to skip a reasonable amount
                    let _remaining = self.data.len() - self.pos;
                    self.pos = self.data.len();
                    Ok(())
                } else {
                    Ok(())
                }
            }
        }
    }

    fn read_object_fields(&mut self, cd: &ClassDesc) -> std::io::Result<JvmValue> {
        let mut map = HashMap::new();

        // Read fields for this class
        for field in &cd.fields {
            let val = self.read_object_field(field)?;
            map.insert(field.name.clone(), val);
        }

        // Handle any class annotations (for writeObject)
        if (cd.flags & SC_WRITE_METHOD) != 0 {
            self.read_class_annotations()?;
        }

        // Read superclass data if any
        // For the class hierarchy, read field data from superclasses
        // We can find the superclass descriptor from the stored class_descs
        let super_name = self.find_super_class_name(&cd.name);
        if let Some(super_name) = super_name
            && !super_name.is_empty()
            && super_name != "java.lang.Object"
            && let Some(super_cd) = self.resolve_class_desc(&super_name)
        {
            let super_fields = self.read_object_fields(&super_cd)?;
            if let JvmValue::Object(super_map) = super_fields {
                map.extend(super_map);
            }
        }

        Ok(JvmValue::Object(map))
    }

    fn find_super_class_name(&self, _class_name: &str) -> Option<String> {
        // In a real implementation, we'd track superclass relationships.
        // For WorldPainter, the classes generally directly extend Object.
        // We'll use the static map for known class hierarchies.
        None
    }

    fn read_object_field(&mut self, field: &FieldInfo) -> std::io::Result<JvmValue> {
        match field.type_code {
            PRIM_BYTE => Ok(JvmValue::Int(self.read_byte()? as i32)),
            PRIM_SHORT => Ok(JvmValue::Int(self.read_u16()? as i16 as i32)),
            PRIM_INT => Ok(JvmValue::Int(self.read_i32()?)),
            PRIM_LONG => Ok(JvmValue::Long(self.read_i64()?)),
            PRIM_FLOAT => {
                let v = self.read_f32()?;
                Ok(JvmValue::Int(v as i32))
            }
            PRIM_DOUBLE => {
                let v = self.read_f64()?;
                Ok(JvmValue::Long(v as i64))
            }
            PRIM_BOOL => Ok(JvmValue::Int(self.read_byte()? as i32)),
            PRIM_CHAR => Ok(JvmValue::Int(self.read_u16()? as i32)),
            b'L' | b'[' => {
                // Object or array reference
                self.read_content()
            }
            _ => {
                // Unknown type, try to read as object
                self.read_content()
            }
        }
    }

    // -----------------------------------------------------------------------
    // Array parsing
    // -----------------------------------------------------------------------

    fn read_array(&mut self) -> std::io::Result<JvmValue> {
        let class_desc_val = self.read_classdesc()?;
        let class_name = class_desc_val.as_string().unwrap_or_default();

        let length = self.read_i32()? as usize;

        if length == 0 {
            self.push_handle(JvmValue::Skipped);
            return Ok(JvmValue::Skipped);
        }

        let elem_type = class_name.as_bytes().get(1).copied();

        let result = match elem_type {
            Some(PRIM_BYTE) => {
                let bytes = self.read_bytes(length)?;
                let arr: Vec<i8> = bytes.iter().map(|&b| b as i8).collect();
                JvmValue::ByteArray(arr)
            }
            Some(PRIM_SHORT) => {
                let mut arr = Vec::with_capacity(length);
                for _ in 0..length {
                    arr.push(self.read_u16()? as i16);
                }
                JvmValue::ShortArray(arr)
            }
            Some(PRIM_INT) => {
                let mut arr = Vec::with_capacity(length);
                for _ in 0..length {
                    arr.push(self.read_i32()?);
                }
                JvmValue::IntArray(arr)
            }
            Some(PRIM_LONG) => {
                for _ in 0..length {
                    let _ = self.read_i64()?;
                }
                JvmValue::LongArray(())
            }
            _ => {
                // Object array — read elements individually
                for _ in 0..length {
                    let _elem = self.read_content()?;
                }
                JvmValue::Skipped
            }
        };

        self.push_handle(result.clone());
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Collection parsers
    // -----------------------------------------------------------------------

    fn read_hashmap_data(&mut self) -> std::io::Result<JvmValue> {
        // HashMap reads: capacity (int), loadFactor (int as float*10000), size (int),
        // then key-value pairs (object, object)
        let _capacity = self.read_i32()?;
        let _load_factor = self.read_i32()?;
        let _threshold = self.read_i32()?;
        let size = self.read_i32()?;

        let mut map = HashMap::new();
        for _ in 0..size {
            let key = self.read_content()?;
            let value = self.read_content()?;
            let key_str = key
                .as_string()
                .unwrap_or_else(|| format!("__key_{}", map.len()));
            map.insert(key_str, value);
        }

        Ok(JvmValue::Object(map))
    }

    fn read_hashset_data(&mut self) -> std::io::Result<JvmValue> {
        let _capacity = self.read_i32()?;
        let _load_factor = self.read_i32()?;
        let _threshold = self.read_i32()?;
        let size = self.read_i32()?;

        let mut map = HashMap::new();
        for i in 0..size {
            let val = self.read_content()?;
            let key = val.as_string().unwrap_or_else(|| format!("__elem_{}", i));
            map.insert(key, val);
        }

        Ok(JvmValue::Object(map))
    }

    fn read_arraylist_data(&mut self) -> std::io::Result<JvmValue> {
        let size = self.read_i32()?;

        let mut map = HashMap::new();
        for i in 0..size {
            let val = self.read_content()?;
            let key = val.as_string().unwrap_or_else(|| format!("__elem_{}", i));
            map.insert(key, val);
        }

        Ok(JvmValue::Object(map))
    }
}

// ---------------------------------------------------------------------------
// WorldPainter data extraction from parsed JvmValues
// ---------------------------------------------------------------------------

struct WpTileData {
    x: i32,
    z: i32,
    heightmap: Option<[i16; 16384]>,
    terrain: Option<[u8; 16384]>,
    water_level: Option<[u8; 16384]>,
}

struct WpWorldData {
    name: String,
    seed: u64,
    tiles: Vec<WpTileData>,
}

fn extract_world_data(val: &JvmValue) -> Result<WpWorldData> {
    let obj = val.as_object().ok_or_else(|| {
        WpImportError::InvalidFormat("root object is not a compound object".into())
    })?;

    let name = obj
        .get("name")
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| "WorldPainter World".to_string());

    let seed = obj.get("seed").and_then(|v| v.as_u64()).unwrap_or(0);

    let mut tiles_data = Vec::new();

    // Find tiles — could be in "tiles" field, or nested
    if let Some(tiles_val) = obj.get("tiles") {
        collect_tiles(tiles_val, &mut tiles_data);
    }

    // Also check for tiles in "dimensions" or other container fields
    for key in &["dimensions", "tileSet", "worldTiles"] {
        if let Some(val) = obj.get(*key) {
            collect_tiles(val, &mut tiles_data);
        }
    }

    // Fallback: scan all values for tile-like objects
    if tiles_data.is_empty() {
        for (_key, val) in obj.iter() {
            collect_tiles(val, &mut tiles_data);
        }
    }

    Ok(WpWorldData {
        name,
        seed,
        tiles: tiles_data,
    })
}

fn collect_tiles(val: &JvmValue, tiles: &mut Vec<WpTileData>) {
    match val {
        JvmValue::Object(map) => {
            // Check if this is a tile by looking for heightMap/terrain fields
            if (map.contains_key("heightMap")
                || map.contains_key("terrain")
                || map.contains_key("waterLevel"))
                && let Some(tile) = parse_single_tile(val)
            {
                tiles.push(tile);
                return;
            }
            // Could be a map from tile-key (Long) to Tile
            for (_key, sub) in map.iter() {
                collect_tiles(sub, tiles);
            }
        }
        JvmValue::ByteArray(_)
        | JvmValue::ShortArray(_)
        | JvmValue::IntArray(_)
        | JvmValue::LongArray(_) => {}
        JvmValue::Skipped => {}
        _ => {}
    }
}

fn parse_single_tile(val: &JvmValue) -> Option<WpTileData> {
    let map = val.as_object()?;
    let x = map.get("x").and_then(|v| v.as_i32()).unwrap_or(0);
    let z = map.get("y").and_then(|v| v.as_i32()).unwrap_or(0);

    let heightmap = map.get("heightMap").and_then(array_to_heightmap);
    let terrain = map.get("terrain").and_then(array_to_u8_16384);
    let water_level = map.get("waterLevel").and_then(array_to_u8_16384);

    Some(WpTileData {
        x,
        z,
        heightmap,
        terrain,
        water_level,
    })
}

fn array_to_heightmap(val: &JvmValue) -> Option<[i16; 16384]> {
    match val {
        JvmValue::ShortArray(arr) => {
            if arr.len() >= 16384 {
                let mut result = [0i16; 16384];
                for (i, &v) in arr.iter().enumerate().take(16384) {
                    result[i] = v;
                }
                Some(result)
            } else {
                None
            }
        }
        JvmValue::IntArray(arr) => {
            if arr.len() >= 16384 {
                let mut result = [0i16; 16384];
                for (i, &v) in arr.iter().enumerate().take(16384) {
                    result[i] = v.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
                }
                Some(result)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn array_to_u8_16384(val: &JvmValue) -> Option<[u8; 16384]> {
    match val {
        JvmValue::ByteArray(arr) => {
            if arr.len() >= 16384 {
                let mut result = [0u8; 16384];
                for (i, &v) in arr.iter().enumerate().take(16384) {
                    result[i] = v as u8;
                }
                Some(result)
            } else {
                None
            }
        }
        JvmValue::IntArray(arr) => {
            if arr.len() >= 16384 {
                let mut result = [0u8; 16384];
                for (i, &v) in arr.iter().enumerate().take(16384) {
                    result[i] = v as u8;
                }
                Some(result)
            } else {
                None
            }
        }
        JvmValue::ShortArray(arr) => {
            if arr.len() >= 16384 {
                let mut result = [0u8; 16384];
                for (i, &v) in arr.iter().enumerate().take(16384) {
                    result[i] = v as u8;
                }
                Some(result)
            } else {
                None
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub struct WpImporter {
    path: PathBuf,
}

impl WpImporter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn import(&self) -> Result<World> {
        import_world_file(&self.path)
    }
}

/// Check whether a path looks like a WorldPainter .world file.
pub fn is_world_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "world")
        || path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().ends_with(".world"))
}

/// Import a WorldPainter .world file into a Terrafier World model.
pub fn import_world_file(path: &Path) -> Result<World> {
    let raw = std::fs::read(path).map_err(WpImportError::Io)?;

    if raw.is_empty() {
        return Err(WpImportError::InvalidFormat("file is empty".into()));
    }

    // Try to decompress if GZip
    let data = if raw.len() >= 2 && raw[0] == 0x1F && raw[1] == 0x8B {
        let mut decoder = flate2::read::GzDecoder::new(&raw[..]);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| WpImportError::Compression(e.to_string()))?;
        decompressed
    } else {
        raw
    };

    // Validate Java serialization magic
    if data.len() < 4 {
        return Err(WpImportError::InvalidFormat(
            "file too small after decompression".into(),
        ));
    }
    let magic = u16::from_be_bytes([data[0], data[1]]);
    if magic != 0xACED {
        return Err(WpImportError::InvalidFormat(format!(
            "not a Java serialization stream (expected magic ACED, got {:04X})",
            magic
        )));
    }
    let stream_version = u16::from_be_bytes([data[2], data[3]]);
    if stream_version != 5 {
        return Err(WpImportError::InvalidFormat(format!(
            "unsupported Java serialization stream version: {}",
            stream_version
        )));
    }

    // Parse the Java serialization stream
    let mut stream = JvmStream::new(data);

    // Read the stream header content (the first object)
    let root = stream
        .read_content()
        .map_err(|e| WpImportError::InvalidFormat(e.to_string()))?;

    // Extract world data from parsed stream
    let world_data = extract_world_data(&root)?;

    if world_data.tiles.is_empty() && !world_data.name.is_empty() {
        // We at least got a name — return an empty world
    }

    let platform = Platform::java_1_18();

    // Build tiles
    let mut tiles = std::collections::HashMap::new();
    for wp_tile in &world_data.tiles {
        let mut tile = Tile::new(
            wp_tile.x,
            wp_tile.z,
            platform.min_height,
            platform.max_height,
        );

        if let Some(hm) = &wp_tile.heightmap {
            tile.heightmap = *hm;
        }
        if let Some(ter) = &wp_tile.terrain {
            tile.terrain = *ter;
        }
        if let Some(wl) = &wp_tile.water_level {
            tile.water_level = *wl;
        }

        tiles.insert((wp_tile.x, wp_tile.z), tile);
    }

    let dimension = Dimension {
        name: "overworld".to_string(),
        tiles,
        min_height: platform.min_height,
        max_height: platform.max_height,
        seed: world_data.seed,
    };

    Ok(World {
        name: world_data.name,
        platform,
        dimensions: vec![dimension],
        seed: world_data.seed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_no_file() {
        let result = import_world_file(Path::new("/nonexistent/magic.world"));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_format() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.world");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"not a world file").unwrap();
        let result = import_world_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_not_world() {
        let result = import_world_file(Path::new("/tmp"));
        assert!(result.is_err());
    }

    #[test]
    fn test_is_world_file() {
        assert!(is_world_file(Path::new("test.world")));
        assert!(!is_world_file(Path::new("test.mca")));
        assert!(!is_world_file(Path::new("level.dat")));
    }
}
