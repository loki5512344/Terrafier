use clap::Args;
use std::path::PathBuf;
use terrafier_core::World;

use crate::util::output::{self, OutputFormat};
use crate::util::progress;

#[derive(Args)]
pub struct NewArgs {
    /// World name
    pub name: String,
    /// Minecraft seed
    #[arg(long, default_value = "0")]
    pub seed: u64,
    /// Output directory
    #[arg(short, long, default_value = ".")]
    pub output: PathBuf,
}

pub fn cmd_new(
    args: NewArgs,
    format: &OutputFormat,
    dry_run: bool,
    validate: bool,
) -> anyhow::Result<()> {
    log::info!("Creating new world: {} (seed: {})", args.name, args.seed);

    if validate || dry_run {
        if args.name.is_empty() {
            anyhow::bail!("World name cannot be empty");
        }
        output::print_result(
            format,
            &serde_json::json!({
                "status": "validated",
                "name": args.name,
                "seed": args.seed,
            }),
        );
        if dry_run {
            return Ok(());
        }
    }

    let spinner = progress::new_spinner("Generating world...");
    let world = World::new(&args.name, args.seed);
    progress::finish_with("World generated", spinner);

    let world_dir = args.output.join(&args.name);
    std::fs::create_dir_all(&world_dir)?;

    let world_json = serde_json::to_string_pretty(&world)?;
    std::fs::write(world_dir.join("world.tfw"), &world_json)?;

    let dim = &world.dimensions[0];
    log::info!("World saved to: {}", world_dir.display());
    output::print_result(
        format,
        &serde_json::json!({
            "status": "created",
            "name": args.name,
            "seed": args.seed,
            "path": world_dir.display().to_string(),
            "tiles": dim.tiles.len(),
        }),
    );

    Ok(())
}
