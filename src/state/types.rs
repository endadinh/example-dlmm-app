use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "ok")]
    Success,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "failure")]
    Failure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebJsonResponse {
    pub status: Status,
    pub message: String,
    pub data: serde_json::Value,
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
    pub destination_mint: String,
    pub amount_in: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub symbol: String,
    pub mint: Pubkey,
    pub decimals: u8,
}
