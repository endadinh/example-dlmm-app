use std::sync::Arc;

use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{account::Account, pubkey::Pubkey};
#[derive(Clone, Deserialize, Serialize)]
pub struct MintAccount {
    pub key: Pubkey,
    pub account: Account,
}

impl MintAccount {
    pub fn fetch(client: Arc<RpcClient>, mint_key: Pubkey) -> Self {
        let account = client
            .get_account(&mint_key)
            .expect("Failed to get mint account");

        MintAccount {
            key: mint_key,
            account,
        }
    }
}
