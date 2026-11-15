//! Local AI deployment: pick a GGUF model for the hardware, download it from
//! HuggingFace, and launch it via llama.cpp's `llama-server`.

pub mod catalog;
pub mod deploy;

pub use catalog::{ModelSpec, recommend};
pub use deploy::{LocalDeployment, LocalServer, deploy};
