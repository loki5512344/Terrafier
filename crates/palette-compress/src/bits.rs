/// Compact storage of fixed-size non-negative integers in a `Vec<i64>`.
///
/// Minecraft Anvil stores block palette indices as packed integers.
/// N bits per entry, entries packed consecutively into 64-bit longs.
/// Entries never cross long boundaries.
pub struct BitArray {
    pub data: Vec<i64>,
    pub bits_per_entry: u8,
    pub size: usize,
}

impl BitArray {
    /// Create a new `BitArray` with all entries initialized to 0.
    pub fn new(size: usize, bits_per_entry: u8) -> Self {
        assert!(
            bits_per_entry > 0 && bits_per_entry <= 64,
            "bits_per_entry must be 1..=64"
        );
        let entries_per_long = 64 / bits_per_entry as usize;
        let data_len = size.div_ceil(entries_per_long);
        BitArray {
            data: vec![0i64; data_len],
            bits_per_entry,
            size,
        }
    }

    /// Wrap existing raw data as a `BitArray`.
    pub fn from_raw(data: Vec<i64>, bits_per_entry: u8, size: usize) -> Self {
        BitArray {
            data,
            bits_per_entry,
            size,
        }
    }

    /// Read the value at `index`.
    pub fn get(&self, index: usize) -> i64 {
        assert!(index < self.size, "index out of bounds");
        let epb = 64 / self.bits_per_entry as usize;
        let long_idx = index / epb;
        let offset = (index % epb) * self.bits_per_entry as usize;
        let mask = (1i64 << self.bits_per_entry) - 1;
        (self.data[long_idx] >> offset) & mask
    }

    /// Write `value` at `index`.
    pub fn set(&mut self, index: usize, value: i64) {
        assert!(index < self.size, "index out of bounds");
        let epb = 64 / self.bits_per_entry as usize;
        let long_idx = index / epb;
        let offset = (index % epb) * self.bits_per_entry as usize;
        let mask = (1i64 << self.bits_per_entry) - 1;
        self.data[long_idx] =
            (self.data[long_idx] & !(mask << offset)) | ((value & mask) << offset);
    }

    /// Borrow the raw underlying long array.
    pub fn to_raw(&self) -> &[i64] {
        &self.data
    }

    /// Minimum bits needed to represent `count` distinct values.
    ///
    /// Minecraft requires at least 1 bit per entry, even for a single block.
    pub fn bits_needed(count: usize) -> u8 {
        if count <= 2 {
            return 1; // 0 and 1 both fit in 1 bit; 2 values → 1 bit
        }
        let count = count as u64;
        (64 - count.saturating_sub(1).leading_zeros()) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack() {
        let mut ba = BitArray::new(10, 4);
        for i in 0..10 {
            ba.set(i, (i % 15) as i64);
        }
        for i in 0..10 {
            assert_eq!(ba.get(i), (i % 15) as i64);
        }
    }

    #[test]
    fn test_bits_needed() {
        assert_eq!(BitArray::bits_needed(0), 1);
        assert_eq!(BitArray::bits_needed(1), 1);
        assert_eq!(BitArray::bits_needed(2), 1);
        assert_eq!(BitArray::bits_needed(3), 2);
        assert_eq!(BitArray::bits_needed(4), 2);
        assert_eq!(BitArray::bits_needed(8), 3);
        assert_eq!(BitArray::bits_needed(16), 4);
        assert_eq!(BitArray::bits_needed(256), 8);
    }

    #[test]
    fn test_large_array() {
        let mut ba = BitArray::new(4096, 13);
        for i in 0..4096 {
            ba.set(i, (i % 8191) as i64);
        }
        for i in 0..4096 {
            assert_eq!(ba.get(i), (i % 8191) as i64);
        }
    }
}
