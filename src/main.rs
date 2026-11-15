use clap::{Parser, Subcommand};
use prjctnatalie::{Config, cli, server};

#[derive(Parser, Debug)]
#[command(
    name = "natalie",
    version,
    about = "Natalie — an AI agent, named after her maintainer."
)]
struct Cli {
    /// Subcommand to run. Defaults to `chat`.
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Interactive chat REPL.
    Chat,
    /// Run the HTTP API server.
    Serve {
        /// Host to bind to.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to bind to.
        #[arg(long, default_value_t = 8080)]
        port: u16,
    },
    /// Show detected hardware and a recommended local model.
    Hardware,
    /// Deploy a local AI via llama.cpp (downloads a GGUF from HuggingFace).
    DeployLocalAi {
        /// Optional model name/filename to force instead of auto-pick.
        #[arg(long)]
        model: Option<String>,
    },
    /// Print version and exit.
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let config = Config::from_env();

    match cli.command.unwrap_or(Commands::Chat) {
        Commands::Chat => cli::run_chat(config).await,
        Commands::Serve { host, port } => server::run(config, host, port).await,
        Commands::Hardware => cli::run_hardware().await,
        Commands::DeployLocalAi { model } => cli::run_deploy_local_ai(model).await,
        Commands::Version => {
            println!("natalie {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}
