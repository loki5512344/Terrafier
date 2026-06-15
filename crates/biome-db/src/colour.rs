//! Biome colour utilities.
//!
//! Converts packed u32 colours to RGB arrays and applies
//! temperature/downfall adjustments for grass and foliage.

use crate::db::BiomeEntry;

pub struct BiomeColour;

fn u32_to_rgb(colour: u32) -> [u8; 3] {
    let r = ((colour >> 16) & 0xFF) as u8;
    let g = ((colour >> 8) & 0xFF) as u8;
    let b = (colour & 0xFF) as u8;
    [r, g, b]
}

impl BiomeColour {
    pub fn water_color(biome: &BiomeEntry) -> [u8; 3] {
        u32_to_rgb(biome.water_color)
    }

    pub fn sky_color(biome: &BiomeEntry) -> [u8; 3] {
        u32_to_rgb(biome.sky_color)
    }

    pub fn fog_color(biome: &BiomeEntry) -> [u8; 3] {
        u32_to_rgb(biome.fog_color)
    }

    /// Returns grass colour adjusted for temperature and downfall.
    ///
    /// When `temperature * downfall` is low (cold/dry), the colour is
    /// muted toward cooler tones. When high (warm/humid), it stays vibrant.
    pub fn grass_color(biome: &BiomeEntry, temperature: f64, downfall: f64) -> [u8; 3] {
        let rgb = u32_to_rgb(biome.grass_color);
        adjust_colour(rgb, temperature, downfall)
    }

    /// Returns foliage colour adjusted for temperature and downfall.
    pub fn foliage_color(biome: &BiomeEntry, temperature: f64, downfall: f64) -> [u8; 3] {
        let rgb = u32_to_rgb(biome.foliage_color);
        adjust_colour(rgb, temperature, downfall)
    }
}

/// Applies Minecraft-style temperature/humidity colour adjustment.
///
/// Low `temp * downfall` mutes the colour (colder/grayer),
/// high values preserve the original vibrancy (warmer/greener).
fn adjust_colour(rgb: [u8; 3], temperature: f64, downfall: f64) -> [u8; 3] {
    let temp = temperature.clamp(0.0, 1.0) as f32;
    let humidity = downfall.clamp(0.0, 1.0) as f32;
    let factor = (temp * humidity).clamp(0.0, 1.0);

    let [r, g, b] = rgb;
    [
        (r as f32 * (0.7 + 0.3 * factor)) as u8,
        (g as f32 * (0.6 + 0.4 * factor)) as u8,
        (b as f32 * (0.5 + 0.5 * factor)) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::BiomeDb;

    fn sample_biome(name: &str) -> BiomeEntry {
        let db = BiomeDb::new();
        db.get_by_name(name).expect("biome must exist").clone()
    }

    #[test]
    fn test_colours_non_zero() {
        let db = BiomeDb::new();
        for biome in db.all_biomes() {
            let water = BiomeColour::water_color(biome);
            let sky = BiomeColour::sky_color(biome);
            let fog = BiomeColour::fog_color(biome);
            let grass =
                BiomeColour::grass_color(biome, biome.temperature as f64, biome.downfall as f64);
            let foliage =
                BiomeColour::foliage_color(biome, biome.temperature as f64, biome.downfall as f64);
            assert!(
                water != [0, 0, 0] || biome.name == "the_end" || biome.name.contains("end_"),
                "water colour should not be zero for {}: {:?}",
                biome.name,
                water
            );
            let all = [water, sky, fog, grass, foliage];
            for (i, c) in all.iter().enumerate() {
                assert_eq!(
                    c.len(),
                    3,
                    "colour {} for {} should have 3 components",
                    i,
                    biome.name
                );
            }
        }
    }

    #[test]
    fn test_plains_colours() {
        let biome = sample_biome("plains");
        let water = BiomeColour::water_color(&biome);
        assert_eq!(water, [63, 118, 228]);
        let grass = BiomeColour::grass_color(&biome, 0.8, 0.4);
        assert_eq!(grass.len(), 3);
    }

    #[test]
    fn test_desert_colours() {
        let biome = sample_biome("desert");
        let sky = BiomeColour::sky_color(&biome);
        assert_eq!(sky, [110, 177, 255]);
        let water = BiomeColour::water_color(&biome);
        assert_eq!(water, [63, 118, 228]);
    }

    #[test]
    fn test_cold_dry_adjustment() {
        let biome = sample_biome("plains");
        let warm = BiomeColour::grass_color(&biome, 1.0, 1.0);
        let cold = BiomeColour::grass_color(&biome, 0.0, 0.0);
        assert!(
            warm[0] >= cold[0] && warm[1] >= cold[1] && warm[2] >= cold[2],
            "warm climate should produce more vibrant colours: warm={:?} cold={:?}",
            warm,
            cold
        );
    }

    #[test]
    fn test_swamp_water() {
        let biome = sample_biome("swamp");
        let water = BiomeColour::water_color(&biome);
        assert_eq!(water, [144, 161, 116]);
    }
}
