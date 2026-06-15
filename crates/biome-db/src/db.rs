//! Biome database with Minecraft 1.21 biome data.
//! Loaded from embedded JSON via `include_str!`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeEntry {
    pub name: String,
    pub id: i32,
    pub temperature: f32,
    pub downfall: f32,
    pub precipitation: String,
    pub water_color: u32,
    pub sky_color: u32,
    pub grass_color: u32,
    pub foliage_color: u32,
    pub fog_color: u32,
}

pub struct BiomeDb {
    by_name: HashMap<String, BiomeEntry>,
    by_id: HashMap<i32, BiomeEntry>,
}

impl BiomeDb {
    pub fn new() -> Self {
        let data = include_str!("../data/biomes.json");
        let entries: Vec<BiomeEntry> =
            serde_json::from_str(data).expect("biomes.json must be valid JSON");
        let mut by_name = HashMap::new();
        let mut by_id = HashMap::new();
        for entry in entries {
            by_name.insert(entry.name.clone(), entry.clone());
            by_id.insert(entry.id, entry);
        }
        Self { by_name, by_id }
    }

    pub fn get_by_name(&self, name: &str) -> Option<&BiomeEntry> {
        self.by_name.get(name)
    }

    pub fn get_by_id(&self, id: i32) -> Option<&BiomeEntry> {
        self.by_id.get(&id)
    }

    pub fn all_biomes(&self) -> impl Iterator<Item = &BiomeEntry> {
        let mut ids: Vec<_> = self.by_id.keys().copied().collect();
        ids.sort();
        ids.into_iter().map(move |id| &self.by_id[&id])
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

impl Default for BiomeDb {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_biomes() {
        let db = BiomeDb::new();
        assert!(!db.is_empty(), "biome db should not be empty");
        assert!(db.len() > 60, "expected 60+ biomes, got {}", db.len());
    }

    #[test]
    fn test_lookup_by_name() {
        let db = BiomeDb::new();
        let plains = db.get_by_name("plains").expect("plains should exist");
        assert_eq!(plains.id, 1);
        assert_eq!(plains.temperature, 0.8);

        let desert = db.get_by_name("desert").expect("desert should exist");
        assert_eq!(desert.id, 2);
        assert_eq!(desert.precipitation, "none");

        let ocean = db.get_by_name("ocean").expect("ocean should exist");
        assert_eq!(ocean.id, 0);
    }

    #[test]
    fn test_lookup_by_id() {
        let db = BiomeDb::new();
        let plains = db.get_by_id(1).expect("id 1 should be plains");
        assert_eq!(plains.name, "plains");

        let desert = db.get_by_id(2).expect("id 2 should be desert");
        assert_eq!(desert.name, "desert");
    }

    #[test]
    fn test_unknown_biome() {
        let db = BiomeDb::new();
        assert!(db.get_by_name("nonexistent").is_none());
        assert!(db.get_by_id(9999).is_none());
    }

    #[test]
    fn test_all_biomes_iter() {
        let db = BiomeDb::new();
        let count = db.all_biomes().count();
        assert_eq!(count, db.len());
    }
}
