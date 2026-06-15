//! Export pipeline — write Terrafier world to Minecraft save or image.

use std::path::Path;
use thiserror::Error;

use crate::io::minecraft::MinecraftIOError;
use crate::model::world::World;

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("Minecraft I/O error: {0}")]
    MinecraftIo(#[from] MinecraftIOError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Render error: {0}")]
    Render(String),
    #[error("Unsupported export format: {0}")]
    UnsupportedFormat(String),
}

pub type Result<T> = std::result::Result<T, ExportError>;

/// Export a Terrafier World to a Minecraft save directory.
pub fn export_to_save(world: &World, output_path: &Path) -> Result<()> {
    crate::io::minecraft::save_world(world, output_path)?;
    Ok(())
}

/// Render a Terrafier world to a PNG image (top-down view).
pub fn render_to_image(world: &World, output_path: &Path, scale: u32) -> Result<()> {
    // Determine bounds from tiles
    let mut min_tx = i32::MAX;
    let mut max_tx = i32::MIN;
    let mut min_tz = i32::MAX;
    let mut max_tz = i32::MIN;

    for dim in &world.dimensions {
        for &(tx, tz) in dim.tiles.keys() {
            min_tx = min_tx.min(tx);
            max_tx = max_tx.max(tx);
            min_tz = min_tz.min(tz);
            max_tz = max_tz.max(tz);
        }
    }

    if min_tx > max_tx || min_tz > max_tz {
        return Err(ExportError::Render("No tiles to render".to_string()));
    }

    let tile_size = 128u32;
    let width = ((max_tx - min_tx + 1) as u32) * tile_size;
    let height = ((max_tz - min_tz + 1) as u32) * tile_size;

    if width == 0 || height == 0 {
        return Err(ExportError::Render(
            "Empty world, nothing to render".to_string(),
        ));
    }

    let img_width = (width / scale).max(1);
    let img_height = (height / scale).max(1);

    let mut img = image::RgbImage::new(img_width, img_height);

    // Terrain color mapping
    let terrain_colors: [image::Rgb<u8>; 7] = [
        image::Rgb([194, 178, 128]), // Desert
        image::Rgb([124, 189, 107]), // Grass
        image::Rgb([86, 140, 74]),   // Forest
        image::Rgb([128, 128, 128]), // Rock
        image::Rgb([227, 212, 160]), // Sand
        image::Rgb([72, 107, 75]),   // Swamp
        image::Rgb([64, 128, 255]),  // Water
    ];

    for dim in &world.dimensions {
        for (&(tx, tz), tile) in &dim.tiles {
            let px = ((tx - min_tx) as u32) * tile_size;
            let pz = ((tz - min_tz) as u32) * tile_size;

            for lx in 0..tile_size as usize {
                for lz in 0..tile_size as usize {
                    let terrain_idx = tile.terrain[lz * tile_size as usize + lx] as usize;
                    let color = terrain_colors[terrain_idx.min(6)];
                    let sx = (px + lx as u32) / scale;
                    let sy = (pz + lz as u32) / scale;

                    if sx < img_width && sy < img_height {
                        let h = tile.heightmap[lz * tile_size as usize + lx];
                        let height_factor = 0.7 + 0.3 * ((h + 64) as f32 / 384.0);
                        let r = (color[0] as f32 * height_factor).min(255.0) as u8;
                        let g = (color[1] as f32 * height_factor).min(255.0) as u8;
                        let b = (color[2] as f32 * height_factor).min(255.0) as u8;
                        img.put_pixel(sx, sy, image::Rgb([r, g, b]));
                    }
                }
            }
        }
    }

    img.save(output_path)
        .map_err(|e| ExportError::Render(e.to_string()))?;

    Ok(())
}

/// Validate world is ready for export.
pub fn validate_export(world: &World) -> Result<()> {
    if world.dimensions.is_empty() {
        return Err(ExportError::Render("World has no dimensions".to_string()));
    }
    for dim in &world.dimensions {
        if dim.tiles.is_empty() {
            log::warn!("Dimension '{}' has no tiles", dim.name);
        }
    }
    Ok(())
}
