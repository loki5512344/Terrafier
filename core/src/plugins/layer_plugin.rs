use crate::model::layers::Layer;

/// A plugin that provides custom layers for world painting.
pub trait LayerPlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn layers(&self) -> Vec<Box<dyn Layer>>;
}
