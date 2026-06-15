//! Import pipeline — load Minecraft saves or Terrafier formats into the world model.

use std::path::Path;
use thiserror::Error;

use crate::io::minecraft::MinecraftIOError;
use crate::model::world::World;

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("Minecraft I/O error: {0}")]
    MinecraftIo(#[from] MinecraftIOError),
    #[error("Unsupported import format: {0}")]
    UnsupportedFormat(String),
    #[error("Import validation failed: {0}")]
    ValidationFailed(String),
}

pub type Result<T> = std::result::Result<T, ImportError>;

/// Import a Minecraft save directory into a Terrafier World model.
pub fn import_minecraft_save(path: &Path) -> Result<World> {
    let world = crate::io::minecraft::load_save(path)?;
    Ok(world)
}

/// Validate a Minecraft save directory before importing.
pub fn validate_save(path: &Path) -> Result<()> {
    if !path.is_dir() {
        return Err(ImportError::ValidationFailed(format!(
            "Path is not a directory: {}",
            path.display()
        )));
    }

    let level_dat = path.join("level.dat");
    if !level_dat.exists() {
        return Err(ImportError::ValidationFailed(
            "Missing level.dat".to_string(),
        ));
    }

    let region_dir = path.join("region");
    if !region_dir.is_dir() {
        return Err(ImportError::ValidationFailed(
            "Missing region directory".to_string(),
        ));
    }

    let region_count = std::fs::read_dir(&region_dir)
        .map_err(|e| ImportError::ValidationFailed(e.to_string()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "mca"))
        .count();

    if region_count == 0 {
        return Err(ImportError::ValidationFailed(
            "No .mca region files found".to_string(),
        ));
    }

    Ok(())
}

/// Import with automatic format detection.
pub fn import(path: &Path) -> Result<World> {
    let path_str = path.display().to_string();

    if path.is_dir() {
        if path.join("level.dat").exists() && path.join("region").is_dir() {
            return import_minecraft_save(path);
        }
    }

    Err(ImportError::UnsupportedFormat(path_str))
}
