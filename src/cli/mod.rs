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
