use crate::model::layers::{DataSize, LAYER_BIOME};
use crate::model::tile::Tile;

mod noise;
mod terrain;

pub use noise::{CavesGenerator, FrostGenerator, RiverGenerator};
pub use terrain::{BiomeGenerator, ResourcesGenerator, TreesGenerator};

pub trait LayerGenerator: Send + Sync {
    fn layer_id(&self) -> u32;
    fn generate(&self, tile: &mut Tile, seed: u64);
}

pub fn generate_all_layers(tile: &mut Tile, seed: u64) {
    let biome_data = terrain::biome::from_terrain(&tile.terrain);
    let buf = tile.ensure_layer(LAYER_BIOME, DataSize::Byte);
    if let crate::model::tile::LayerBuffer::Byte(bytes) = buf {
        bytes.copy_from_slice(&biome_data);
    }

    noise::CavesGenerator.generate(tile, seed);
    noise::RiverGenerator.generate(tile, seed);
    noise::FrostGenerator.generate(tile, seed);
    terrain::TreesGenerator.generate(tile, seed);
    terrain::ResourcesGenerator.generate(tile, seed);
}

#[cfg(test)]
mod tests {
    use crate::model::layers::{
        LAYER_BIOME, LAYER_CAVES, LAYER_FROST, LAYER_RESOURCES, LAYER_RIVER, LAYER_TREES,
    };
    use crate::model::tile::{LayerBuffer, Tile};

    fn make_test_tile(height: i16, terrain: u8) -> Tile {
        let mut tile = Tile::new(0, 0, -64, 320);
        for h in tile.heightmap.iter_mut() {
            *h = height;
        }
        for t in tile.terrain.iter_mut() {
            *t = terrain;
        }
        tile
    }

    #[test]
    fn test_generate_all_layers_creates_all_buffers() {
        let mut tile = make_test_tile(70, 1);
        super::generate_all_layers(&mut tile, 42);

        assert!(tile.get_layer_data(LAYER_CAVES).is_some());
        assert!(tile.get_layer_data(LAYER_RIVER).is_some());
        assert!(tile.get_layer_data(LAYER_FROST).is_some());
        assert!(tile.get_layer_data(LAYER_TREES).is_some());
        assert!(tile.get_layer_data(LAYER_BIOME).is_some());
        assert!(tile.get_layer_data(LAYER_RESOURCES).is_some());
    }

    #[test]
    fn test_biome_layer_matches_terrain() {
        let mut tile = Tile::new(0, 0, -64, 320);
        let terrain_map: [u8; 7] = [0, 1, 2, 3, 4, 5, 6];
        for (i, t) in tile.terrain.iter_mut().enumerate() {
            *t = terrain_map[i % 7];
        }

        super::generate_all_layers(&mut tile, 42);
        let data = tile.get_layer_data(LAYER_BIOME).unwrap();
        if let LayerBuffer::Byte(b) = data {
            assert_eq!(b[0], 0, "desert → plains");
            assert_eq!(b[1], 0, "grass → plains");
            assert_eq!(b[2], 2, "forest → forest");
            assert_eq!(b[3], 3, "rock → taiga");
            assert_eq!(b[4], 1, "sand → desert");
            assert_eq!(b[5], 4, "swamp → swamp");
            assert_eq!(b[6], 8, "water → ocean");
        } else {
            panic!("biome layer should be Byte");
        }
    }

    #[test]
    fn test_frost_above_snow_line() {
        let mut tile = make_test_tile(95, 1);
        super::generate_all_layers(&mut tile, 42);

        let data = tile.get_layer_data(LAYER_FROST).unwrap();
        if let LayerBuffer::Bit(bits) = data {
            for &word in bits.iter() {
                assert_eq!(word, u64::MAX);
            }
        } else {
            panic!("frost layer should be Bit");
        }
    }

    #[test]
    fn test_frost_below_snow_line() {
        let mut tile = make_test_tile(50, 1);
        super::generate_all_layers(&mut tile, 42);

        let data = tile.get_layer_data(LAYER_FROST).unwrap();
        if let LayerBuffer::Bit(bits) = data {
            for &word in bits.iter() {
                assert_eq!(word, 0);
            }
        } else {
            panic!("frost layer should be Bit");
        }
    }

    #[test]
    fn test_river_water_terrain_no_river() {
        let mut tile = make_test_tile(63, 6);
        super::generate_all_layers(&mut tile, 42);

        let data = tile.get_layer_data(LAYER_RIVER).unwrap();
        if let LayerBuffer::Byte(bytes) = data {
            for &b in bytes.iter() {
                assert_eq!(b, 0);
            }
        } else {
            panic!("river layer should be Byte");
        }
    }

    #[test]
    fn test_trees_forest_terrain_has_trees() {
        let mut tile = make_test_tile(70, 2);
        super::generate_all_layers(&mut tile, 42);

        let data = tile.get_layer_data(LAYER_TREES).unwrap();
        if let LayerBuffer::Byte(bytes) = data {
            assert!(bytes.iter().any(|&b| b > 0));
        } else {
            panic!("trees layer should be Byte");
        }
    }

    #[test]
    fn test_caves_no_caves_in_water() {
        let mut tile = make_test_tile(-10, 6);
        super::generate_all_layers(&mut tile, 42);

        let data = tile.get_layer_data(LAYER_CAVES).unwrap();
        if let LayerBuffer::Nibble(nibbles) = data {
            for &n in nibbles.iter() {
                assert_eq!(n, 0);
            }
        } else {
            panic!("caves layer should be Nibble");
        }
    }

    #[test]
    fn test_caves_in_rock_terrain() {
        let mut tile = make_test_tile(60, 3);
        super::generate_all_layers(&mut tile, 42);

        let data = tile.get_layer_data(LAYER_CAVES).unwrap();
        if let LayerBuffer::Nibble(nibbles) = data {
            assert!(nibbles.iter().any(|&n| n > 0));
        } else {
            panic!("caves layer should be Nibble");
        }
    }

    #[test]
    fn test_resources_layer_is_empty() {
        let mut tile = make_test_tile(70, 3);
        super::generate_all_layers(&mut tile, 42);

        let data = tile.get_layer_data(LAYER_RESOURCES).unwrap();
        if let LayerBuffer::Nibble(nibbles) = data {
            for &n in nibbles.iter() {
                assert_eq!(n, 0);
            }
        } else {
            panic!("resources layer should be Nibble");
        }
    }
}
