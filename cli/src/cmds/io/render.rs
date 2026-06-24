use clap::Args;
use std::path::PathBuf;

use crate::util::output::{self, OutputFormat};
use crate::util::{load_world, progress};

#[derive(Args)]
pub struct RenderArgs {
    /// Path to Terrafier world directory
    pub world: String,
    /// Output image path
    #[arg(short, long, default_value = "preview.png")]
    pub output: PathBuf,
    /// Scale factor (1 = full resolution, 2 = half, 4 = quarter)
    #[arg(long, default_value = "4")]
    pub scale: u32,
}

pub fn cmd_render(
    args: RenderArgs,
    format: &OutputFormat,
    dry_run: bool,
    validate: bool,
) -> anyhow::Result<()> {
    let world_path = std::path::Path::new(&args.world);
    log::info!("Rendering world: {}", world_path.display());

    let world = load_world(world_path)?;
    terrafier_core::io::export::validate_export(&world)?;

    let tile_count: usize = world.dimensions.iter().map(|d| d.tiles.len()).sum();

    if validate || dry_run {
        output::print_result(
            format,
            &serde_json::json!({
                "status": "validated",
                "world": args.world,
                "tiles": tile_count,
            }),
        );
        if dry_run {
            return Ok(());
        }
    }

    let bar = progress::new_bar(tile_count as u64, "Rendering...");
    terrafier_core::io::export::render_to_image(&world, &args.output, args.scale)?;
    progress::finish_with("Render complete", bar);

    log::info!("Preview rendered to: {}", args.output.display());
    output::print_result(
        format,
        &serde_json::json!({
            "status": "rendered",
            "world": args.world,
            "output": args.output.display().to_string(),
            "scale": args.scale,
        }),
    );

    Ok(())
}
