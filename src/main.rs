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
            let mut config = app::AppConfig::default();

            if let Ok(rpc_url) = dotenv::var("RPC_URL") {
                info!("Using RPC URL from .env: {}", rpc_url);
                config.rpc_url = rpc_url;
            }

            if let Ok(pool_ttl_secs) = dotenv::var("POOL_CACHE_TTL_SECS") {
                if let Ok(pool_ttl) = pool_ttl_secs.parse::<u64>() {
                    info!("Using Pool Cache TTL from .env: {} seconds", pool_ttl);
                    config.cache_ttl.pool_ttl = Duration::from_secs(pool_ttl);
                }
            }

            if let Ok(token_ttl_secs) = dotenv::var("TOKEN_CACHE_TTL_SECS") {
                if let Ok(token_ttl) = token_ttl_secs.parse::<u64>() {
                    info!("Using Token Cache TTL from .env: {} seconds", token_ttl);
                    config.cache_ttl.token_ttl = Duration::from_secs(token_ttl);
                }
            }

            if let Ok(bin_ttl_secs) = dotenv::var("BIN_CACHE_TTL_SECS") {
                if let Ok(bin_ttl) = bin_ttl_secs.parse::<u64>() {
                    info!("Using Bin Cache TTL from .env: {} seconds", bin_ttl);
                    config.cache_ttl.bin_ttl = Duration::from_secs(bin_ttl);
                }
            }
            if web {
                web::start_web_server(config).await?;
            } else {
                info!("Running in CLI-only mode...");
            }
        }
    }

    Ok(())
}
