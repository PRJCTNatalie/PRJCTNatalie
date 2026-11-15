//! Command-line interface and interactive REPL.

use crate::agent::Agent;
use crate::config::Config;
use std::io::{self, BufRead, Write};

pub async fn run_chat(config: Config) -> anyhow::Result<()> {
    if !config.has_credentials() {
        eprintln!(
            "warning: OPENAI_API_KEY is not set. Requests will likely fail unless your \
             backend does not require authentication."
        );
    }

    let mut agent = Agent::new(config);
    println!("Natalie agent. Type a message and press Enter.");
    println!("Commands: /reset (clear history), /tools (list tools), /exit (quit).");

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("input error: {e}");
                break;
            }
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "/exit" || trimmed == "/quit" {
            break;
        }
        if trimmed == "/reset" {
            agent.reset();
            println!("(history cleared)");
            continue;
        }
        if trimmed == "/tools" {
            println!("{}", agent.tools().names().join(", "));
            continue;
        }

        print!("natalie> ");
        let _ = stdout.flush();
        match agent.chat(trimmed).await {
            Ok(reply) => println!("{reply}"),
            Err(e) => eprintln!("error: {e}"),
        }
    }
    Ok(())
}

pub async fn run_hardware() -> anyhow::Result<()> {
    let hw = crate::HardwareInfo::detect();
    println!("CPU cores: {}", hw.cpu_cores);
    println!("Total RAM: {} MB", hw.total_ram_mb);
    match &hw.gpu {
        Some(g) => println!("GPU: {} ({} MB VRAM, backend: {})", g.name, g.vram_mb, g.backend),
        None => println!("GPU: none detected (CPU-only)"),
    }
    println!("Model budget: {} MB", hw.model_budget_mb());
    println!();
    println!("Fitting models:");
    for m in crate::localai::catalog::fitting(&hw) {
        println!("  - {} ({} MB) {}", m.name, m.size_mb, m.description);
    }
    if let Some(m) = crate::localai::catalog::recommend(&hw) {
        println!();
        println!("Recommended: {} — {}", m.name, m.description);
    } else {
        println!();
        println!("No model fits this hardware.");
    }
    Ok(())
}

pub async fn run_deploy_local_ai(model: Option<String>) -> anyhow::Result<()> {
    use crate::localai::deploy;
    let (deployment, _server) = deploy::deploy(model.as_deref()).await?;
    println!();
    println!("Local AI deployed.");
    println!("  Model:    {}", deployment.model);
    println!("  Endpoint: {}", deployment.endpoint);
    println!("  Model file: {}", deployment.model_path.display());
    println!();
    println!("To use it with Natalie, set:");
    println!("  export OPENAI_BASE_URL={}/v1", deployment.endpoint);
    println!("  export OPENAI_API_KEY=local");
    println!("  export NATALIE_MODEL=local");
    println!();
    println!("Keeping llama-server running. Press Ctrl+C to stop.");
    // Keep process alive.
    tokio::signal::ctrl_c().await?;
    println!("Stopping local AI.");
    Ok(())
}
