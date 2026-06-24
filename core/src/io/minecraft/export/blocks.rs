//! Block name mapping — terrain type → Minecraft block IDs.

pub(crate) const BLOCK_SET: &[&str] = &[
    "minecraft:air",
    "minecraft:grass_block",
    "minecraft:dirt",
    "minecraft:stone",
    "minecraft:bedrock",
    "minecraft:sand",
    "minecraft:sandstone",
    "minecraft:water",
    "minecraft:deepslate",
];

pub(crate) const BLOCK_SET_EXTENDED: &[&str] = &[
    "minecraft:air",
    "minecraft:grass_block",
    "minecraft:dirt",
    "minecraft:stone",
    "minecraft:bedrock",
    "minecraft:sand",
    "minecraft:sandstone",
    "minecraft:water",
    "minecraft:deepslate",
    "minecraft:snow_block",
    "minecraft:ice",
    "minecraft:oak_log",
    "minecraft:coal_ore",
    "minecraft:iron_ore",
    "minecraft:copper_ore",
    "minecraft:gold_ore",
    "minecraft:redstone_ore",
    "minecraft:lapis_ore",
    "minecraft:diamond_ore",
    "minecraft:emerald_ore",
];

/// Minimum number of bits needed to represent values up to n-1 (clamped to MC minimum 4).
pub(crate) fn bits_needed(n: usize) -> usize {
    if n <= 1 {
        return 4;
    }
    let bits = (usize::BITS - (n - 1).leading_zeros()) as usize;
    bits.max(4)
}

/// Pack palette indices into a compact long array (Minecraft block state format).
pub(crate) fn pack_indices(indices: &[u16], bits: usize) -> Vec<i64> {
    if indices.is_empty() || bits == 0 {
        return Vec::new();
    }
    let total_bits = indices.len() * bits;
    let longs = total_bits.div_ceil(64);
    let mut data = vec![0i64; longs];
    let mask = (1i64 << bits) - 1;
    for (i, &idx) in indices.iter().enumerate() {
        let bit_pos = i * bits;
        let long_idx = bit_pos / 64;
        let bit_offset = bit_pos % 64;
        data[long_idx] |= (idx as i64 & mask) << bit_offset;
    }
    data
}

pub(crate) fn block_name(terrain: u8, y: i32, surface_y: i32) -> &'static str {
    if terrain == 6 {
        if y <= surface_y {
            return "minecraft:water";
        } else {
            return "minecraft:air";
        }
    }

    if y == surface_y {
        match terrain {
            1 | 4 => "minecraft:sand",
            3 => "minecraft:stone",
            _ => "minecraft:grass_block",
        }
    } else if y > surface_y {
        "minecraft:air"
    } else if y > surface_y - 4 {
        match terrain {
            1 | 4 => "minecraft:sand",
            3 => "minecraft:stone",
            _ => "minecraft:dirt",
        }
    } else if y < -60 {
        "minecraft:bedrock"
    } else if y < 0 {
        "minecraft:deepslate"
    } else {
        "minecraft:stone"
    }
}
