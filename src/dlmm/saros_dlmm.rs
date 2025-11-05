use std::time::Duration;

use anyhow::Result;
use saros_sdk::{
    math::{
        fees::{
            compute_transfer_amount_for_expected_output, compute_transfer_fee, TokenTransferFee,
        },
        swap_manager::{get_swap_result, SwapType},
    },
    state::{
        bin_array::{BinArray, BinArrayPair},
        pair::Pair,
    },
    utils::helper::{find_event_authority, get_hook_bin_array, get_pair_bin_array, is_swap_for_y},
};

use serde_json::{json, Value};
use solana_client::rpc_client::RpcClient;
use solana_program::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use tokio::{sync::RwLock, time::Instant};

use crate::state::{PairAccount, PoolState, QuoteParams, QuoteResponse, SwapMode};

pub struct DLMMClient {
    pub program_id: Pubkey,
    pub key: Pubkey,
    pub label: String,
    pub pair: Pair,
    pub token_transfer_fee: TokenTransferFee,
    pub bin_array_lower: BinArray,
    pub bin_array_upper: BinArray,
    pub bin_array_key: [Pubkey; 2],
    pub token_vault: [Pubkey; 2],
    pub token_program: [Pubkey; 2],
    pub event_authority: Pubkey,
    pub hook: Pubkey,
    // // Remaining accounts of the LB program cpi call to hooks, will be checked at hook program.
    pub active_hook_bin_array_key: [Pubkey; 2],
    pub last_updated: Instant,

    // pub pool_key: Pubkey,
    // pub pool_state: RwLock<Option<Pair>>,
    // pub bins: RwLock<Vec<Bin>>,
    pub pool_state: RwLock<Option<PoolState>>,
    pub last_updateds: RwLock<Instant>,
    pub need_update: RwLock<bool>,
    // pub rpc: Arc<RpcClient>,
}

impl DLMMClient {
    pub fn new_from_pair(pair_account: PairAccount) -> Result<Self>
    where
        Self: Sized,
    {
        let account_data = &pair_account.account.data[..];
        let pair = Pair::unpack(account_data)?;

        let bin_array_index = pair.bin_array_index();

        let (bin_array_lower_key, bin_array_upper_key) = get_pair_bin_array(
            bin_array_index,
            &pair_account.key,
            &pair_account.account.owner,
        );

        let (mut active_hook_bin_array_lower_key, mut active_hook_bin_array_upper_key) =
            (Pubkey::default(), Pubkey::default());

        let mut hook_key = pair_account.key; // Dummy key if no hook

        if let Some(pair_hook_key) = pair.hook {
            (
                active_hook_bin_array_lower_key,
                active_hook_bin_array_upper_key,
            ) = get_hook_bin_array(bin_array_index, pair_hook_key);

            hook_key = pair_hook_key;
        }

        let event_authority = find_event_authority(pair_account.account.owner);

        Ok(Self {
            program_id: pair_account.account.owner,
            key: pair_account.key,
            label: "saros_dlmm".into(),
            pair: pair.clone(),
            token_transfer_fee: TokenTransferFee::default(),
            bin_array_lower: BinArray::default(),
            bin_array_upper: BinArray::default(),
            bin_array_key: [bin_array_lower_key, bin_array_upper_key],
            token_vault: [Pubkey::default(), Pubkey::default()],
            token_program: [Pubkey::default(), Pubkey::default()],
            event_authority,
            hook: hook_key,
            active_hook_bin_array_key: [
                active_hook_bin_array_lower_key,
                active_hook_bin_array_upper_key,
            ],
            last_updated: Instant::now(),
            pool_state: RwLock::new(None),
            last_updateds: RwLock::new(Instant::now() - Duration::from_secs(9999)),
            need_update: RwLock::new(true),
        })
    }

    pub fn key(&self) -> Pubkey {
        self.key
    }

    pub fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        vec![
            self.key,
            self.bin_array_key[0],
            self.bin_array_key[1],
            self.pair.token_mint_x,
            self.pair.token_mint_y,
        ]
    }

    // fn update(&mut self, account_map: &AccountMap) -> Result<()> {
    //     // update pair
    //     let pair_account = account_map
    //         .get(&self.key)
    //         .ok_or_else(|| anyhow::anyhow!("Pair account not found"))?;
    //     self.pair = Pair::unpack(&pair_account.data)?;

    //     // update bin arrays
    //     let bin_array_lower_account = account_map
    //         .get(&self.bin_array_key[0])
    //         .ok_or_else(|| anyhow::anyhow!("Bin array lower account not found"))?;
    //     self.bin_array_lower = BinArray::unpack(&bin_array_lower_account.data)?;

    //     let bin_array_upper_account = account_map
    //         .get(&self.bin_array_key[1])
    //         .ok_or_else(|| anyhow::anyhow!("Bin array upper account not found"))?;
    //     self.bin_array_upper = BinArray::unpack(&bin_array_upper_account.data)?;

    //     Ok(())

    //  };

    pub fn quote(&self, quote_params: &QuoteParams) -> Result<QuoteResponse> {
        let QuoteParams {
            amount,
            swap_mode,
            input_mint,
        } = *quote_params;
        let mut pair = self.pair.clone();

        let bin_array = BinArrayPair::merge(self.bin_array_lower, self.bin_array_upper)?;

        let block_timestamp = u64::try_from(self.last_updated.elapsed().as_secs()).unwrap_or(0);
        let swap_for_y = is_swap_for_y(input_mint, self.pair.token_mint_x);

        let (mint_in, epoch_transfer_fee_in, epoch_transfer_fee_out) = if swap_for_y {
            (
                self.pair.token_mint_x,
                self.token_transfer_fee.epoch_transfer_fee_x,
                self.token_transfer_fee.epoch_transfer_fee_y,
            )
        } else {
            (
                self.pair.token_mint_y,
                self.token_transfer_fee.epoch_transfer_fee_y,
                self.token_transfer_fee.epoch_transfer_fee_x,
            )
        };

        let (amount_in, amount_out, fee_amount) = match swap_mode {
            SwapMode::ExactIn => {
                let (amount_in_after_transfer_fee, _) =
                    compute_transfer_fee(epoch_transfer_fee_in, amount)?;

                let (amount_out, fee_amount) = get_swap_result(
                    &mut pair,
                    bin_array,
                    amount_in_after_transfer_fee,
                    swap_for_y,
                    SwapType::ExactIn,
                    block_timestamp,
                )?;

                let (amount_out_after_transfer_fee, _) =
                    compute_transfer_fee(epoch_transfer_fee_out, amount_out)?;

                (amount, amount_out_after_transfer_fee, fee_amount)
            }
            SwapMode::ExactOut => {
                let (amount_out_before_transfer_fee, _) =
                    compute_transfer_amount_for_expected_output(epoch_transfer_fee_out, amount)?;

                let (amount_in, fee_amount) = get_swap_result(
                    &mut pair,
                    bin_array,
                    amount_out_before_transfer_fee,
                    swap_for_y,
                    SwapType::ExactOut,
                    block_timestamp,
                )?;

                let (amount_in_before_transfer_fee, _) =
                    compute_transfer_amount_for_expected_output(epoch_transfer_fee_in, amount_in)?;

                let (amount_out_after_transfer_fee, _) =
                    compute_transfer_fee(epoch_transfer_fee_out, amount)?;

                (
                    amount_in_before_transfer_fee,
                    amount_out_after_transfer_fee,
                    fee_amount,
                )
            }
        };

        Ok(QuoteResponse {
            in_amount: amount_in,
            out_amount: amount_out,
            fee_amount,
            fee_mint: mint_in,
            ..Default::default()
        })
    }

    pub async fn refresh_from_chain(&mut self, client: &RpcClient) -> Result<()> {
        let account = client.get_account(&self.key)?;
        let pair = Pair::unpack(&account.data)?;

        self.pair = pair;
        self.program_id = account.owner;
        self.last_updated = Instant::now();

        Ok(())
    }

    pub async fn read_or_refresh_pool(&self) -> Result<PoolState> {
        {
            let guard = self.pool_state.read().await;
            if guard.is_some() {
                return Ok(guard.clone().unwrap());
            }
        }
        // if pool not loaded, refresh it
        // self.maybe_refresh().await?;
        let guard = self.pool_state.read().await;
        guard
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Pool load failed"))
    }

    // async fn fetch_from_rpc(&self) -> Result<(PoolState, Vec<Bin>)> {
    //     // fetch & deserialize pool and bins from on-chain accounts
    //     // Ok((PoolState {}, vec![]))

    //     Ok((PoolState {}, vec![]))
    // }

    // /// Get a quote for a given input amount
    // pub async fn get_quote(&self, amount_in: u64) -> anyhow::Result<f64> {
    //     {
    //         let pool_guard = self.pool_state.read().await;
    //         if pool_guard.is_none() {
    //             drop(pool_guard); // thả read lock trước khi gọi write
    //             self.maybe_refresh(Duration::ZERO).await?;
    //         }
    //     }

    //     let pool_guard = self.pool_state.read().await;
    //     let pool = pool_guard
    //         .clone()
    //         .ok_or_else(|| anyhow::anyhow!("Pool not loaded"))?;

    //     let bins_guard = self.bins.read().await;
    //     let bins = bins_guard.clone();

    //     // tính quote
    //     Ok(amount_in as f64 * 0.99)
    // }
}

#[async_trait::async_trait]
pub trait DLMMClientInterface {
    // Define methods for the DLMMClient trait here
}

#[async_trait::async_trait]
pub trait DLMMClientService
where
    Self: Send + Sync,
{
    // Define methods for the DLMMClientService trait here
    async fn list_pools(&self) -> Result<Value>;
    async fn get_pool_info(&self, pool: &str) -> Result<Value>;
    async fn get_quote(&self, req: QuoteParams) -> Result<Value>;
    async fn simulate_swap(&self, amount_in: u64) -> Result<Value>;
}
#[async_trait::async_trait]
impl DLMMClientService for DLMMClient {
    async fn list_pools(&self) -> Result<Value> {
        // mock: replace with real fetch later
        Ok(json!([
            { "id": "SoL-SAROS", "liquidity": 12345, "fee": "0.3%" },
            { "id": "USDC-SOL", "liquidity": 9999, "fee": "0.05%" }
        ]))
    }

    async fn get_pool_info(&self, pool: &str) -> Result<Value> {
        Ok(json!({
            "id": pool,
            "token_a": "So11111111111111111111111111111111111111112",
            "token_b": "SAROS1111111111111111111111111111111111111",
            "fee_rate": "0.3%",
            "liquidity": "123456.78",
        }))
    }

    async fn get_quote(&self, req: QuoteParams) -> Result<Value> {
        let quote_response = self.quote(&req)?;

        Ok(json!({
            "input_amount": quote_response.in_amount,
            "output_amount": quote_response.out_amount,
            "fee": quote_response.fee_amount,
            "fee_mint": quote_response.fee_mint.to_string(),
        }))
    }

    async fn simulate_swap(&self, amount_in: u64) -> Result<Value> {
        Ok(json!({
            "simulated_output": amount_in * 97 / 100,
            "estimated_slippage": "0.2%"
        }))
    }
}
