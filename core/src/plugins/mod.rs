//! Plugin host — trait-объекты для расширения функциональности.
//!
//! Плагины позволяют добавлять custom слои, операции редактирования,
//! форматы экспорта и источники генерации, не меняя ядро.

pub mod export_plugin;
pub mod layer_plugin;
pub mod operation_plugin;
pub mod registry;
pub mod source_plugin;

pub use export_plugin::ExportPlugin;
pub use layer_plugin::LayerPlugin;
pub use operation_plugin::OperationPlugin;
pub use registry::PluginRegistry;
pub use source_plugin::TileSourcePlugin;
