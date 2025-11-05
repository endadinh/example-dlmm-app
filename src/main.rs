mod app;
mod cli;
mod dlmm;
mod state;
mod web;

use std::time::Duration;

use clap::Parser;
use cli::{Cli, Commands};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start { web } => {
            info!("🚀 Starting Saros DLMM Interface...");

            let rpc_url = dotenv::var("RPC_URL")
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

            let cache_ttl_secs: u64 = dotenv::var("CACHE_TTL_SECS")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .unwrap_or(15);

            let config = app::AppConfig {
                rpc_url,
                cache_ttl: app::TTLConfig {
                    pool_ttl: Duration::from_secs(cache_ttl_secs),
                    token_ttl: Duration::from_secs(3600),
                    bin_ttl: Duration::from_secs(10),
                },
            };

            if web {
                web::start_web_server(config).await?;
            } else {
                info!("Running in CLI-only mode...");
            }
        }
    }

    Ok(())
}
