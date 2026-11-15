//! Hardware detection for choosing a local model.

use serde::{Deserialize, Serialize};

/// Detected GPU, if any.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    /// VRAM in megabytes (0 if unknown).
    pub vram_mb: u64,
    /// Backend hint: "cuda", "rocm", "metal", "vulkan", or "cpu".
    pub backend: String,
}

/// Summary of the machine's capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu_cores: usize,
    pub total_ram_mb: u64,
    pub gpu: Option<GpuInfo>,
}

impl HardwareInfo {
    /// Best-effort detection of the current machine.
    pub fn detect() -> Self {
        let cpu_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        let total_ram_mb = detect_ram_mb().unwrap_or(0);
        let gpu = detect_gpu();

        Self {
            cpu_cores,
            total_ram_mb,
            gpu,
        }
    }

    /// Effective memory available for a model: VRAM if a GPU is present,
    /// otherwise system RAM (leaving ~2 GiB headroom).
    pub fn model_budget_mb(&self) -> u64 {
        if let Some(g) = &self.gpu {
            if g.vram_mb > 0 {
                return g.vram_mb.saturating_sub(512);
            }
        }
        self.total_ram_mb.saturating_sub(2048)
    }

    pub fn is_gpu(&self) -> bool {
        self.gpu.as_ref().map(|g| g.vram_mb > 0).unwrap_or(false)
    }
}

fn detect_ram_mb() -> Option<u64> {
    // /proc/meminfo (Linux)
    let content = std::fs::read_to_string("/proc/meminfo").ok()?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            let kb: u64 = rest.trim().split_whitespace().next()?.parse().ok()?;
            return Some(kb / 1024);
        }
    }
    None
}

fn detect_gpu() -> Option<GpuInfo> {
    // NVIDIA
    if let Ok(out) = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total", "--format=csv,noheader,nounits"])
        .output()
    {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = text.lines().next() {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 2 {
                    let name = parts[0].to_string();
                    let vram_mb = parts[1].parse::<u64>().unwrap_or(0);
                    return Some(GpuInfo {
                        name,
                        vram_mb,
                        backend: "cuda".into(),
                    });
                }
            }
        }
    }

    // AMD (rocm-smi)
    if let Ok(out) = std::process::Command::new("rocm-smi")
        .args(["--showmeminfo", "vram", "--json"])
        .output()
    {
        if out.status.success() {
            // Best-effort: try to find a VRAM total in the JSON-ish output.
            let text = String::from_utf8_lossy(&out.stdout);
            // Fallback: parse first number followed by "MB" or just report presence.
            // We keep it simple: report a GPU with unknown VRAM -> 0 (caller treats as CPU).
            // Attempt to extract a number.
            let vram_mb = text
                .split_whitespace()
                .filter_map(|t| t.trim_matches(|c: char| !c.is_ascii_digit()).parse::<u64>().ok())
                .max()
                .unwrap_or(0);
            return Some(GpuInfo {
                name: "AMD GPU".into(),
                vram_mb,
                backend: "rocm".into(),
            });
        }
    }

    // macOS Metal (not expected on this Linux box, but harmless).
    #[cfg(target_os = "macos")]
    {
        // sysctl not parsed here; leave as None.
    }

    None
}
