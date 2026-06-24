//! Plugin host — trait-объекты для расширения функциональности.

mod traits;
pub mod registry;

pub use traits::{ExportPlugin, LayerPlugin, OperationPlugin, TileSourcePlugin};
pub use registry::PluginRegistry;
