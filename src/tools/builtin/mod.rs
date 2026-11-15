//! Built-in tools shipped with the agent.

pub mod echo;
pub mod time;
pub mod fs;

pub use echo::EchoTool;
pub use time::TimeTool;
pub use fs::{ReadFileTool, ListDirTool};

use crate::tools::registry::ToolRegistry;
use std::sync::Arc;

/// Register all built-in tools into a registry.
pub fn register_all(registry: &mut ToolRegistry) {
    registry.register(Arc::new(EchoTool));
    registry.register(Arc::new(TimeTool));
    registry.register(Arc::new(ReadFileTool));
    registry.register(Arc::new(ListDirTool));
}
