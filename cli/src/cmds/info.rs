use clap::Args;
use std::path::Path;

use crate::util::load_world;
use crate::util::output::{self, OutputFormat};

#[derive(Args)]
pub struct InfoArgs {
    /// Path to Terrafier world or Minecraft save
    pub world: String,
}

pub fn cmd_info(
    args: InfoArgs,
    format: &OutputFormat,
    _dry_run: bool,
    _validate: bool,
) -> anyhow::Result<()> {
    let world_path = Path::new(&args.world);

    let world = match load_world(world_path) {
        Ok(w) => w,
        Err(_) => terrafier_core::io::import::import(world_path)?,
    };

    let tile_count: usize = world.dimensions.iter().map(|d| d.tiles.len()).sum();

    output::print_result(
        format,
        &serde_json::json!({
            "name": world.name,
            "seed": world.seed,
            "platform": world.platform.display_name,
            "platform_id": world.platform.id,
            "height_range": {
                "min": world.platform.min_height,
                "max": world.platform.max_height,
            },
            "dimensions": world.dimensions.iter().map(|d| serde_json::json!({
                "name": d.name,
                "tiles": d.tiles.len(),
                "seed": d.seed,
                "min_height": d.min_height,
                "max_height": d.max_height,
            })).collect::<Vec<_>>(),
            "total_tiles": tile_count,
            "path": world_path.display().to_string(),
        }),
    );

    Ok(())
}
