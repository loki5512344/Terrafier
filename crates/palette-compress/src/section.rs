use std::collections::HashMap;
use terrafier_nbt::Tag;

use crate::bits::BitArray;
use crate::palette::{BlockPalette, BlockState};

/// A 16×16×16 chunk section with palette-based block storage.
pub struct SectionData {
    pub palette: BlockPalette,
    pub storage: BitArray,
}

impl SectionData {
    /// Create a new section filled with air.
    pub fn new() -> Self {
        let mut palette = BlockPalette::new();
        let air = BlockState::new("minecraft:air");
        palette.add_or_get(air);
        let bpe = palette.bits_per_entry();
        let storage = BitArray::new(4096, bpe);
        // index 0 = air
        SectionData { palette, storage }
    }

    /// Get the block at the given section-local coordinates.
    ///
    /// Coordinates must be in 0..16.
    pub fn get_block(&self, x: u8, y: u8, z: u8) -> Option<&BlockState> {
        let idx = index(x, y, z);
        let pal_idx = self.storage.get(idx) as u32;
        self.palette.get(pal_idx)
    }

    /// Set the block at the given section-local coordinates.
    ///
    /// Coordinates must be in 0..16.
    pub fn set_block(&mut self, x: u8, y: u8, z: u8, block: BlockState) {
        let idx = index(x, y, z);
        let pal_idx = self.palette.add_or_get(block);
        // Grow storage if bits_per_entry changed
        let new_bpe = self.palette.bits_per_entry();
        if new_bpe != self.storage.bits_per_entry {
            let mut new_storage = BitArray::new(4096, new_bpe);
            for i in 0..4096 {
                let v = self.storage.get(i);
                new_storage.set(i, v);
            }
            self.storage = new_storage;
        }
        self.storage.set(idx, pal_idx as i64);
    }

    /// Fill the entire section with one block type.
    pub fn fill(&mut self, block: BlockState) {
        let pal_idx = self.palette.add_or_get(block);
        let bpe = self.palette.bits_per_entry();
        self.storage = BitArray::new(4096, bpe);
        // Set all entries to pal_idx
        for i in 0..4096 {
            self.storage.set(i, pal_idx as i64);
        }
    }

    /// Number of unique block states in the palette.
    pub fn palette_size(&self) -> usize {
        self.palette.len()
    }

    /// Parse from an NBT `TAG_Compound` representing a section.
    ///
    /// Expects `"BlockStates"` (LongArray) and `"Palette"` (List of Compound).
    pub fn from_nbt(tag: &Tag) -> Option<Self> {
        let compound = match tag {
            Tag::Compound(map) => map,
            _ => return None,
        };
        let block_states = match compound.get("BlockStates")? {
            Tag::LongArray(arr) => arr,
            _ => return None,
        };
        let palette_list = match compound.get("Palette")? {
            Tag::List(list) => list,
            _ => return None,
        };

        let (palette, storage) = BlockPalette::from_nbt(palette_list, block_states);
        Some(SectionData { palette, storage })
    }

    /// Serialize to an NBT `TAG_Compound`.
    pub fn to_nbt(&self) -> Tag {
        let (palette_list, block_states) = self.palette.to_nbt(&self.storage);
        let mut map = HashMap::new();
        map.insert("BlockStates".to_string(), Tag::LongArray(block_states));
        map.insert("Palette".to_string(), Tag::List(palette_list));
        Tag::Compound(map)
    }
}

/// Convert section-local (x, y, z) to linear index.
///
/// Minecraft order: y → z → x
fn index(x: u8, y: u8, z: u8) -> usize {
    (y as usize) * 256 + (z as usize) * 16 + (x as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_section() {
        let sec = SectionData::new();
        assert_eq!(sec.palette_size(), 1);
        assert_eq!(
            sec.get_block(0, 0, 0),
            Some(&BlockState::new("minecraft:air"))
        );
    }

    #[test]
    fn test_set_get_block() {
        let mut sec = SectionData::new();
        let stone = BlockState::new("minecraft:stone");
        sec.set_block(0, 0, 0, stone.clone());
        assert_eq!(sec.get_block(0, 0, 0), Some(&stone));
        assert_eq!(sec.palette_size(), 2);
    }

    #[test]
    fn test_fill() {
        let mut sec = SectionData::new();
        let stone = BlockState::new("minecraft:stone");
        sec.fill(stone.clone());
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    assert_eq!(sec.get_block(x, y, z), Some(&stone));
                }
            }
        }
    }

    #[test]
    fn test_nbt_roundtrip() {
        let mut sec = SectionData::new();
        sec.set_block(1, 2, 3, BlockState::new("minecraft:stone"));
        sec.set_block(
            4,
            5,
            6,
            BlockState::new("minecraft:grass_block").with_property("snowy", "true"),
        );

        let nbt = sec.to_nbt();
        let restored = SectionData::from_nbt(&nbt).unwrap();

        assert_eq!(sec.palette_size(), restored.palette_size());
        assert_eq!(sec.get_block(1, 2, 3), restored.get_block(1, 2, 3));
        assert_eq!(sec.get_block(4, 5, 6), restored.get_block(4, 5, 6));
        assert_eq!(sec.get_block(0, 0, 0), restored.get_block(0, 0, 0));
    }
}
