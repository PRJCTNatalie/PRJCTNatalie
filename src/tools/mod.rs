//! Tooling: trait, registry, and built-in tools.

pub mod builtin;
pub mod registry;

pub use registry::{Tool, ToolError, ToolRegistry};
