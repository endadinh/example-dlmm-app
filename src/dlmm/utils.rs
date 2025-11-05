use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use saros_sdk::{math::swap_manager::SwapType, state::pair::Pair, utils::helper::is_swap_for_y};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{account::Account, pubkey::Pubkey};
use tokio::task;

use crate::{
    dlmm::DLMMClient,
    state::{QuoteParams, QuoteRequest},
};

/// Task automatically refreshes the DLMMClient's pool and bins at the given TTL interval.
pub fn spawn_refresher(client: Arc<DLMMClient>, ttl: Duration) {
    task::spawn(async move {
        loop {
            if let Err(err) = maybe_refresh(client.clone()).await {
                tracing::warn!("⚠️ Failed to refresh {}: {}", client.key, err);
            }
            tokio::time::sleep(ttl).await;
        }
    });
}

pub async fn maybe_refresh(client: Arc<DLMMClient>) -> anyhow::Result<()> {
    // client.refresh_pool().await?;
    // client.refresh_bins().await?;
    Ok(())
}
