use core::fmt;
use mpl_token_metadata::accounts::Metadata;
use saros_sdk::{state::pair::Pair, utils::helper::is_swap_for_y};
use solana_sdk::{packet::Meta, program_pack::Pack, pubkey::Pubkey};
use std::{collections::HashMap, str::FromStr, string, sync::Arc, time::Duration};
use tokio::time::Instant;

use tokio::sync::RwLock;
use tracing::{info, warn};

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::json;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

use solana_client::rpc_client::RpcClient;

use crate::app::{AppConfig, AppContext};
use crate::dlmm;
use crate::state::{QuoteRequest, SwapMode, TokenMeta, TokenResponse};
use crate::{
    dlmm::{DLMMClient, DLMMClientInterface, DLMMClientService},
    state::QuoteParams,
};

pub async fn start_web_server(config: AppConfig) -> anyhow::Result<()> {
    let app_state = Arc::new(AppContext::new(config));

    let static_files = ServeDir::new(format!("{}/web/dist", env!("CARGO_MANIFEST_DIR")));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let public_routes = Router::new().route("/api/network/status", get(ping));

    let sdk_routes = Router::new()
        .route("/api/pair", get(get_pair))
        .route("/api/quote", post(get_quote))
        .route("/api/simulate", post(simulate_swap));

    // Define API routes
    let app = Router::new()
        .merge(public_routes)
        .merge(sdk_routes)
        .route("/api/ping", get(|| async { "pong 🦀" }))
        .fallback_service(static_files)
        .layer(cors)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Web server listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

/// === Handlers ===

async fn ping() -> &'static str {
    "pong 🦀"
}

/// Serde deserialization decorator to map empty Strings to None,
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

/// Get pool info by pubkey
#[axum::debug_handler]
async fn get_pair(
    State(ctx): State<Arc<AppContext>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let pair_address = params.get("address").cloned().unwrap_or_default();
    if pair_address.len() < 20 {
        return Json(json!({ "error": "Invalid address format" }));
    }

    let pair_key = Pubkey::from_str_const(&pair_address);

    // Step 1: Get or create DLMM client
    let dlmm_client = match ctx.get_or_spawn_client(pair_key).await {
        Ok(client) => client,
        Err(e) => {
            return Json(json!({ "error": format!("Failed to get DLMM client: {}", e) }));
        }
    };

    info!("🔍 Fetching metadata from RPC for pair {}", pair_address);

    let [mint_a_meta, mint_b_meta] = match ctx.fetch_pair_token_info(&dlmm_client).await {
        Ok(mints) => mints,
        Err(e) => {
            return Json(json!({ "error": format!("Failed to get token info: {}", e) }));
        }
    };

    Json(json!({
        "status": "ok",
        "token_a": { "symbol": mint_a_meta.symbol, "mint": mint_a_meta.mint.to_string(), "decimals": mint_a_meta.decimals },
        "token_b": { "symbol": mint_b_meta.symbol, "mint": mint_b_meta.mint.to_string(), "decimals": mint_b_meta.decimals },
        "pool_id": pair_address
    }))
}

async fn get_pools(State(_ctx): State<Arc<AppContext>>) -> Json<serde_json::Value> {
    // mock: replace with real fetch later
    let pools = json!([
        { "pool_id": "ExamplePool1", "token_a": "SOL", "token_b": "USDC" },
        { "pool_id": "ExamplePool2", "token_a": "ETH", "token_b": "USDT" }
    ]);

    Json(pools)
}

async fn get_quote(
    State(ctx): State<Arc<AppContext>>,
    Json(body): Json<QuoteRequest>,
) -> Json<serde_json::Value> {
    let pair_address = body.pair_address.clone();

    info!("🔍 Getting quote for pair {}", pair_address);

    info!("Body: {:?}", body);

    let source_mint = Pubkey::from_str_const(&body.source_mint);

    // 1️⃣ take DLMM client
    let dlmm_client = match ctx
        .get_or_spawn_client(Pubkey::from_str_const(&pair_address))
        .await
    {
        Ok(client) => client,
        Err(e) => {
            return Json(json!({ "error": format!("Failed to get DLMM client: {}", e) }));
        }
    };

    tracing::info!(
        "💱 Quoting swap: amount_in={}, source_mint={}",
        body.amount_in,
        body.source_mint
    );

    let is_swap_for_y = is_swap_for_y(source_mint, dlmm_client.pair.token_mint_x);

    let swap_mode = if is_swap_for_y {
        if source_mint == dlmm_client.pair.token_mint_x {
            SwapMode::ExactIn
        } else {
            SwapMode::ExactOut
        }
    } else {
        if source_mint == dlmm_client.pair.token_mint_x {
            SwapMode::ExactOut
        } else {
            SwapMode::ExactIn
        }
    };

    let req = QuoteParams {
        amount: body.amount_in as u64,
        input_mint: Pubkey::from_str_const(&body.source_mint),
        swap_mode,
    };

    // 2️⃣ call get_quote() from DLMM client
    let result = match dlmm_client.quote(&req) {
        Ok(quote) => Json(json!({
            "status": "ok",
            "pair": pair_address,
            "input": body.source_mint,
            "output": body.source_mint,
            "quote": quote
        })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    };

    result

    // let mock_result = Json(json!({
    //     "status": "ok",
    //     "pair": pair_address,
    //     "input": body.source_mint,
    //     "output": body.source_mint,
    //     "quote": {
    //         "amount_in": body.amount_in,
    //         "amount_out": body.amount_in * 98 / 100,
    //         "fee_amount": body.amount_in * 2 / 100
    //     }
    // }));

    // mock_result
}

async fn simulate_swap(
    State(_ctx): State<Arc<AppContext>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let pool_id = body
        .get("pool_id")
        .and_then(|v| v.as_str())
        .unwrap_or("SOL-USDC");
    let amount = body
        .get("amount_in")
        .and_then(|v| v.as_u64())
        .unwrap_or(1u64);

    // mock result
    let result = json!({
        "pool_id": pool_id,
        "amount_in": amount,
        "simulated_output": amount * 98 / 100,
        // "simulation": simulation
    });

    Json(json!({ "result": result }))
}
