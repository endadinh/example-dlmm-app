use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Copy, Clone)]
pub enum SwapMode {
    ExactIn,
    ExactOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub in_amount: u64,
    pub out_amount: u64,
    pub fee_amount: u64,
    pub fee_mint: Pubkey,
}
impl Default for QuoteResponse {
    fn default() -> Self {
        QuoteResponse {
            in_amount: 0,
            out_amount: 0,
            fee_amount: 0,
            fee_mint: Pubkey::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct QuoteRequest {
    pub pair_address: String,
    pub source_mint: String,
    pub amount_in: u64,
}
pub struct QuoteParams {
    pub amount: u64,
    pub input_mint: Pubkey,
    pub swap_mode: SwapMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub symbol: String,
    pub mint: Pubkey,
    pub decimals: u8,
}
