pub mod output;
pub mod progress;

use std::path::Path;
use terrafier_core::World;

/// Load a Terrafier world from JSON format (.tfw).
pub fn load_world(path: &Path) -> anyhow::Result<World> {
    let tfw_path = if path.is_dir() {
        path.join("world.tfw")
    } else {
        path.to_path_buf()
    };
    let content = std::fs::read_to_string(&tfw_path)?;
    let world: World = serde_json::from_str(&content)?;
    Ok(world)
}
