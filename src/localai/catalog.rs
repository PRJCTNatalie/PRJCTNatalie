//! Curated catalog of GGUF models with hardware requirements.

use crate::hardware::HardwareInfo;

/// A candidate local model.
#[derive(Debug, Clone)]
pub struct ModelSpec {
    /// HuggingFace repo id, e.g. "bartowski/TinyLlama-1.1B-Chat-v1.0-GGUF".
    pub repo: &'static str,
    /// GGUF filename within the repo.
    pub filename: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// Approximate download size in MB.
    pub size_mb: u64,
    /// Minimum model budget (MB) required to run.
    pub min_budget_mb: u64,
    /// Whether a GPU is strongly recommended.
    pub needs_gpu: bool,
    /// Short description.
    pub description: &'static str,
}

/// Curated list, ordered from smallest to largest.
pub const CATALOG: &[ModelSpec] = &[
    ModelSpec {
        repo: "bartowski/TinyLlama-1.1B-Chat-v1.0-GGUF",
        filename: "TinyLlama-1.1B-Chat-v1.0-Q4_K_M.gguf",
        name: "TinyLlama 1.1B (Q4_K_M)",
        size_mb: 637,
        min_budget_mb: 1024,
        needs_gpu: false,
        description: "Tiny, fast, CPU-friendly. Good for low-end hardware.",
    },
    ModelSpec {
        repo: "bartowski/Phi-3.1-mini-4k-instruct-GGUF",
        filename: "Phi-3.1-mini-4k-instruct-Q4_K_M.gguf",
        name: "Phi-3.1 Mini 3.8B (Q4_K_M)",
        size_mb: 2300,
        min_budget_mb: 3072,
        needs_gpu: false,
        description: "Strong small model, runs on CPU with 4+ GiB free.",
    },
    ModelSpec {
        repo: "bartowski/Llama-3.2-3B-Instruct-GGUF",
        filename: "Llama-3.2-3B-Instruct-Q4_K_M.gguf",
        name: "Llama 3.2 3B (Q4_K_M)",
        size_mb: 2000,
        min_budget_mb: 3072,
        needs_gpu: false,
        description: "Meta's compact instruct model.",
    },
    ModelSpec {
        repo: "bartowski/Mistral-7B-Instruct-v0.3-GGUF",
        filename: "Mistral-7B-Instruct-v0.3-Q4_K_M.gguf",
        name: "Mistral 7B (Q4_K_M)",
        size_mb: 4370,
        min_budget_mb: 6144,
        needs_gpu: true,
        description: "Solid 7B; prefers a 6+ GiB GPU.",
    },
    ModelSpec {
        repo: "bartowski/Meta-Llama-3.1-8B-Instruct-GGUF",
        filename: "Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf",
        name: "Llama 3.1 8B (Q4_K_M)",
        size_mb: 4920,
        min_budget_mb: 7168,
        needs_gpu: true,
        description: "Strong 8B; prefers an 8+ GiB GPU.",
    },
];

/// Recommend the largest model that fits the hardware.
pub fn recommend(hw: &HardwareInfo) -> Option<&'static ModelSpec> {
    let budget = hw.model_budget_mb();
    let mut best: Option<&ModelSpec> = None;
    for m in CATALOG {
        if m.min_budget_mb > budget {
            continue;
        }
        // Pick the largest that fits.
        match best {
            None => best = Some(m),
            Some(prev) => {
                if m.size_mb > prev.size_mb {
                    best = Some(m);
                }
            }
        }
    }
    best
}

/// List all models that fit.
pub fn fitting(hw: &HardwareInfo) -> Vec<&'static ModelSpec> {
    let budget = hw.model_budget_mb();
    CATALOG
        .iter()
        .filter(|m| m.min_budget_mb <= budget)
        .collect()
}
