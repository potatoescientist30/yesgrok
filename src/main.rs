use std::convert::Infallible;
use clap::{Parser, Subcommand};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::Duration;
use hyper::{Client, Request, Body};
use hyper::client::HttpConnector;
use hyper::header::{CONNECTION, UPGRADE, CONTENT_TYPE};
use hyper::upgrade::Upgraded;
use serde_json::json;
use tokio::signal;

// todo: should this be static?
const AGENT_ID: &str = "128393847599";

#[derive(Parser)]
#[command(name = "yesgrok", version = "0.1", about = "Tunnel agent")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Expose a TCP port
    Tcp { port: u16 },
    /// Expose an HTTP port
    Http { port: u16 },
}

// "dyn" is used since we do not know the exact error type at compile time.
// Instead we have to use a trait object. As long as the error type implements
// the Error trait, everything is ok.
// Box<T> is a pointer to a data storage on the heap. Trait objects do not have a
// known size at compile time, so we have to put them behind a pointer.
#[tokio::main]
async fn main() -> Result<(), Box <dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tcp { port } => {
            println!("✅ Successfully exposed TCP port {}", port);
            // Future: implement TCP tunneling
        }
        Commands::Http { port } => {
            println!("✅ Exposing HTTP port {}", port);
            connect_to_server(port).await?;
        }
    }

    Ok(())
}

// todo: we should sent debug messages to some server to be able to solve user issues.
async fn connect_to_server(port: u16) -> Result<(), Infallible> {
    let client: Client<HttpConnector, Body> = Client::new();

    let register_request_body = json!({
        "agent_id": AGENT_ID
    }).to_string();
    let register_request = Request::builder()
        .method("POST")
        .uri("http://localhost:4000/register")
        .header(CONTENT_TYPE, "application/json")
        .header(CONNECTION, "Upgrade")
        .header(UPGRADE, "Agent-protocol")
        .body(Body::from(register_request_body)).unwrap();

    client.request(register_request).await.expect("❌ Failed to connect to the tunnel server. Please check your internet connection.");

    let body = json!({
        "agent_id": AGENT_ID
    }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("http://localhost:4000/connect")
        .header(CONTENT_TYPE, "application/json")
        .header(CONNECTION, "Upgrade")
        .header(UPGRADE, "Agent-protocol")
        .body(Body::from(body)).unwrap();

    // Send request
    let mut response = client.request(req).await.expect("❌ Failed to connect to the tunnel server. Please check your internet connection.");

    if response.status() == 101 {

            let upgraded: Upgraded = hyper::upgrade::on(&mut response).await.expect("❌ Failed to connect to the tunnel server. Please check your internet connection.");
            let (mut reader, mut writer) = tokio::io::split(upgraded);

            println!("✅ Connected to the tunnel!");

            // Reader from the socket
            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                loop {
                    match reader.read(&mut buf).await {
                        Ok(0) => {
                            eprintln!("❌ Connection to the tunnel lost. Please check your internet connection.");
                            break;
                        }
                        Ok(n) => {
                            println!("📥 From server: {}", String::from_utf8_lossy(&buf[..n]));
                        }
                        Err(e) => {
                            eprintln!("⚠️ Read error: {:?}", e);
                            break;
                        }
                    }
                }
            });

            // Writer to the socket
            tokio::spawn(async move {
                loop {
                    let msg = format!("Ping from agent {} on port {}\n", AGENT_ID, port);
                    if let Err(e) = writer.write_all(msg.as_bytes()).await {
                        eprintln!("⚠️ Write error: {:?}", e);
                        break;
                    }
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            });

        // Quits if the user wants to.
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        println!("👋 Agent shutting down...");

    } else {
        eprintln!("❌ Failed to connect to the tunnel server. Please check your internet connection.");
    }

    Ok(())
}
