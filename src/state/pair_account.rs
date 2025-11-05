use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{account::Account, pubkey::Pubkey};
use tokio::task;
use tracing::info;

use crate::dlmm::DLMMClient;
use anyhow::Result;

#[derive(Clone, Deserialize, Serialize)]
pub struct PairAccount {
    pub key: Pubkey,
    pub account: Account,
}

impl PairAccount {
    pub fn fetch(client: Arc<RpcClient>, pair_key: Pubkey) -> Result<PairAccount> {
        let account = client.get_account(&pair_key)?;

        Ok(PairAccount {
            key: pair_key,
            account,
        })
    }
}
