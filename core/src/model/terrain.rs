use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terrain {
    Desert,
    Grass,
    Forest,
    Rock,
    Sand,
    Swamp,
    Water,
}

impl Terrain {
    pub fn name(&self) -> &'static str {
        match self {
            Terrain::Desert => "Desert",
            Terrain::Grass => "Grass",
            Terrain::Forest => "Forest",
            Terrain::Rock => "Rock",
            Terrain::Sand => "Sand",
            Terrain::Swamp => "Swamp",
            Terrain::Water => "Water",
        }
    }
}
