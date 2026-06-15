use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::model::dimension::Dimension;
use crate::model::platform::Platform;
use crate::ops::heightmap::{HeightMapSource, NoiseHeightMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub name: String,
    pub platform: Platform,
    pub dimensions: Vec<Dimension>,
    pub seed: u64,
}

impl World {
    /// Create a new world with a single overworld dimension and generated terrain.
    pub fn new(name: &str, seed: u64) -> Self {
        let platform = Platform::java_1_18();

        // Generate tiles in parallel using rayon
        let coords: Vec<(i32, i32)> = (-1..=1)
            .flat_map(|tx| (-1..=1).map(move |tz| (tx, tz)))
            .collect();

        let tiles: Vec<(i32, i32, crate::model::tile::Tile)> = coords
            .par_iter()
            .map(|&(tx, tz)| {
                let mut tile =
                    crate::model::tile::Tile::new(tx, tz, platform.min_height, platform.max_height);
                let source = NoiseHeightMap::default();
                source.generate(
                    &mut tile,
                    seed.wrapping_add(
                        (tx.wrapping_mul(374_761_393) as u64)
                            .wrapping_add((tz as u64).wrapping_mul(668_265_263)),
                    ),
                );
                (tx, tz, tile)
            })
            .collect();

        let mut dim_tiles = std::collections::HashMap::with_capacity(tiles.len());
        for (tx, tz, tile) in tiles {
            dim_tiles.insert((tx, tz), tile);
        }

        let dimension = Dimension {
            name: "overworld".to_string(),
            tiles: dim_tiles,
            min_height: platform.min_height,
            max_height: platform.max_height,
            seed,
        };

        Self {
            name: name.to_string(),
            platform,
            dimensions: vec![dimension],
            seed,
        }
    }

    /// Get mutable reference to the overworld dimension.
    pub fn overworld_mut(&mut self) -> Option<&mut Dimension> {
        self.dimensions.iter_mut().find(|d| d.name == "overworld")
    }
}
