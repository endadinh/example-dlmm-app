use std::sync::Arc;

use crate::state::pair_account::PairAccount;
use saros_sdk::state::bin_array::BinArray;
use saros_sdk::state::pair::Pair;
use saros_sdk::utils::helper::get_pair_bin_array;
use solana_client::rpc_client::RpcClient;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Copy)]
pub struct PoolState {
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub active_bin_array_lower: BinArray,
    pub active_bin_array_upper: BinArray,
}

impl PoolState {
    pub fn new(
        mint_x: Pubkey,
        mint_y: Pubkey,
        active_bin_array_lower: BinArray,
        active_bin_array_upper: BinArray,
    ) -> Self {
        PoolState {
            mint_x,
            mint_y,
            active_bin_array_lower,
            active_bin_array_upper,
        }
    }

    pub fn fetch(client: Arc<RpcClient>, pair_account: PairAccount) -> Self {
        let pair_state =
            Pair::unpack(&pair_account.account.data).expect("Failed to unpack pair account");
        let bin_array_index = pair_state.bin_array_index();
        let (bin_array_lower_key, bin_array_upper_key) = get_pair_bin_array(
            bin_array_index,
            &pair_account.key,
            &pair_account.account.owner,
        );

        let bin_array_lower_account = client
            .get_account(&bin_array_lower_key)
            .expect("Failed to get bin array lower account");
        let bin_array_lower = BinArray::unpack(&bin_array_lower_account.data)
            .expect("Failed to unpack bin array lower account");
        let bin_array_upper_account = client
            .get_account(&bin_array_upper_key)
            .expect("Failed to get bin array upper account");
        let bin_array_upper = BinArray::unpack(&bin_array_upper_account.data)
            .expect("Failed to unpack bin array upper account");

        PoolState::new(
            pair_state.token_mint_x,
            pair_state.token_mint_y,
            bin_array_lower,
            bin_array_upper,
        )
    }
}
