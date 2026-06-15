use super::{ExportPlugin, LayerPlugin, OperationPlugin, TileSourcePlugin};

/// Central registry for all plugins.
pub struct PluginRegistry {
    pub layers: Vec<Box<dyn LayerPlugin>>,
    pub operations: Vec<Box<dyn OperationPlugin>>,
    pub exports: Vec<Box<dyn ExportPlugin>>,
    pub sources: Vec<Box<dyn TileSourcePlugin>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            operations: Vec::new(),
            exports: Vec::new(),
            sources: Vec::new(),
        }
    }

    pub fn register_layer(&mut self, plugin: Box<dyn LayerPlugin>) {
        self.layers.push(plugin);
    }

    pub fn register_operation(&mut self, plugin: Box<dyn OperationPlugin>) {
        self.operations.push(plugin);
    }

    pub fn register_export(&mut self, plugin: Box<dyn ExportPlugin>) {
        self.exports.push(plugin);
    }

    pub fn register_source(&mut self, plugin: Box<dyn TileSourcePlugin>) {
        self.sources.push(plugin);
    }

    pub fn all_plugin_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for p in &self.layers {
            names.push(p.name().to_string());
        }
        for p in &self.operations {
            names.push(p.name().to_string());
        }
        for p in &self.exports {
            names.push(p.name().to_string());
        }
        for p in &self.sources {
            names.push(p.name().to_string());
        }
        names
    }

    /// Find a layer plugin by name.
    pub fn find_layer_by_name(&self, name: &str) -> Option<&dyn LayerPlugin> {
        self.layers
            .iter()
            .find(|p| p.name() == name)
            .map(Box::as_ref)
    }

    /// Find an operation plugin by name.
    pub fn find_operation_by_name(&self, name: &str) -> Option<&dyn OperationPlugin> {
        self.operations
            .iter()
            .find(|p| p.name() == name)
            .map(Box::as_ref)
    }

    /// Find an export plugin by name.
    pub fn find_export_by_name(&self, name: &str) -> Option<&dyn ExportPlugin> {
        self.exports
            .iter()
            .find(|p| p.name() == name)
            .map(Box::as_ref)
    }

    /// Find a tile source plugin by name.
    pub fn find_source_by_name(&self, name: &str) -> Option<&dyn TileSourcePlugin> {
        self.sources
            .iter()
            .find(|p| p.name() == name)
            .map(Box::as_ref)
    }
}
