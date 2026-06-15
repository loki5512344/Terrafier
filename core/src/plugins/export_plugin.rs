use crate::model::world::World;
use std::path::Path;

/// A plugin that provides custom export formats.
pub trait ExportPlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn can_export(&self, world: &World) -> bool;
    fn export(&self, world: &World, path: &Path) -> Result<(), Box<dyn std::error::Error>>;
    fn format_name(&self) -> &'static str;
}
