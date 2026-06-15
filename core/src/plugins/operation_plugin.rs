use crate::ops::operations::Operation;

/// A plugin that provides custom editing operations.
pub trait OperationPlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn operations(&self) -> Vec<Box<dyn Fn() -> Box<dyn Operation>>>;
}
