use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "yesgrok", version = "0.1", about = "Tunnel agent")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Expose a TCP port
    Tcp {
        port: u16,
    },
    /// Expose an HTTP port
    Http {
        port: u16,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tcp { port } => {
            println!("✅ Successfully exposed TCP port {}", port);
        }
        Commands::Http { port } => {
            println!("✅ Successfully exposed HTTP port {}", port);
        }
    }
}
