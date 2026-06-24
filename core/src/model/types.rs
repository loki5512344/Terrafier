use serde::{Deserialize, Serialize};

pub trait Brush: Send + Sync {
    fn get_strength(&self, dx: f64, dy: f64) -> f64;
    fn radius(&self) -> f64;
}

pub struct SymmetricBrush {
    pub radius: f64,
}

impl SymmetricBrush {
    pub fn new(radius: f64) -> Self {
        Self { radius }
    }
}

impl Brush for SymmetricBrush {
    fn get_strength(&self, dx: f64, dy: f64) -> f64 {
        let dist = (dx * dx + dy * dy).sqrt();
        if dist >= self.radius {
            0.0
        } else {
            1.0 - (dist / self.radius)
        }
    }
    fn radius(&self) -> f64 {
        self.radius
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub id: String,
    pub display_name: String,
    pub min_height: i16,
    pub max_height: i16,
}

impl Platform {
    pub fn java_1_18() -> Self {
        Self {
            id: "java_anvil_1_18".to_string(),
            display_name: "Minecraft Java 1.18+".to_string(),
            min_height: -64,
            max_height: 320,
        }
    }
}

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
