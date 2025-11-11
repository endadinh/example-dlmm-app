use saros_sdk::utils::helper::is_swap_for_y;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tracing::info;

use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

use crate::{
    app::{AppConfig, AppContext},
    state::{QuoteRequest, Status, WebJsonResponse},
};
use anyhow::Result;

use jupiter_amm_interface::{Amm, QuoteParams, SwapMode};

pub async fn start_web_server(config: AppConfig) -> Result<()> {
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

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Web server listening on http://{}", addr);
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

/// === Handlers ===
async fn ping() -> &'static str {
    "pong 🦀"
}

/// Get pool info by pubkey
#[axum::debug_handler]
async fn get_pair(
    State(ctx): State<Arc<AppContext>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<WebJsonResponse> {
    let pair_address = params.get("address").cloned().unwrap_or_default();
    if pair_address.len() < 20 {
        return Json(WebJsonResponse {
            status: Status::Error,
            message: "Invalid address format".to_string(),
            data: json!({}),
        });
    }

    let pair_key = Pubkey::from_str_const(&pair_address);

    // Step 1: Get or create DLMM client
    let dlmm_client = match ctx.get_or_spawn_client(pair_key).await {
        Ok(client) => client,
        Err(e) => {
            return Json(WebJsonResponse {
                status: Status::Error,
                message: format!("Failed to get DLMM client: {}", e),
                data: json!({}),
            });
        }
    };

    info!("🔍 Fetching metadata from RPC for pair {}", pair_address);
    let saros_dlmm = dlmm_client.saros_dlmm.read().await;

    let [mint_a_meta, mint_b_meta] = match ctx.fetch_pair_token_info(&saros_dlmm).await {
        Ok(mints) => mints,
        Err(e) => {
            return Json(WebJsonResponse {
                status: Status::Error,
                message: format!("Failed to fetch token metadata: {}", e),
                data: json!({}),
            });
        }
    };

    Json(WebJsonResponse {
        status: Status::Success,
        message: "Pair fetched successfully".to_string(),
        data: json!({
            "pair_address": pair_address,
            "token_mint_x": saros_dlmm.pair.token_mint_x.to_string(),
            "token_mint_y": saros_dlmm.pair.token_mint_y.to_string(),
            "token_a": {
                "mint": mint_a_meta.mint.to_string(),
                "symbol": mint_a_meta.symbol,
                "decimals": mint_a_meta.decimals,
            },
            "token_b": {
                "mint": mint_b_meta.mint.to_string(),
                "symbol": mint_b_meta.symbol,
                "decimals": mint_b_meta.decimals,
            },
        }),
    })
}

#[axum::debug_handler]
async fn get_quote(
    State(ctx): State<Arc<AppContext>>,
    Json(body): Json<QuoteRequest>,
) -> Json<WebJsonResponse> {
    let pair_address = body.pair_address.clone();

    info!("🔍 Getting quote for pair {}", pair_address);
    info!("Body: {:?}", body);

    let source_mint = Pubkey::from_str_const(&body.source_mint);
    let destination_mint = Pubkey::from_str_const(&body.destination_mint);

    // 1️⃣ take DLMM client
    let dlmm_client = match ctx
        .get_or_spawn_client(Pubkey::from_str_const(&pair_address))
        .await
    {
        Ok(client) => client,
        Err(e) => {
            return Json(WebJsonResponse {
                status: Status::Error,
                message: format!("Failed to get DLMM client: {}", e),
                data: json!({}),
            });
        }
    };

    tracing::info!(
        "💱 Quoting swap: amount_in={}, source_mint={}",
        body.amount_in,
        body.source_mint
    );

    for _ in 0..3 {
        if let Err(e) = dlmm_client.update(&ctx).await {
            tracing::warn!("⚠️ Failed to update DLMM client: {}", e);
            continue;
        }
    }

    let client = dlmm_client.saros_dlmm.read().await;

    let is_swap_for_y = is_swap_for_y(source_mint, client.pair.token_mint_x);

    let swap_mode = if is_swap_for_y {
        if source_mint == client.pair.token_mint_x {
            SwapMode::ExactIn
        } else {
            SwapMode::ExactOut
        }
    } else {
        if source_mint == client.pair.token_mint_x {
            SwapMode::ExactOut
        } else {
            SwapMode::ExactIn
        }
    };

    let req = QuoteParams {
        amount: body.amount_in,
        input_mint: source_mint,
        swap_mode,
        output_mint: destination_mint,
    };

    // 2️⃣ call get_quote() from DLMM client
    let result = match client.quote(&req) {
        Ok(quote) => Json(WebJsonResponse {
            status: Status::Success,
            message: "quote successful".to_string(),
            data: json!({
                "in_amount": quote.in_amount,
                "out_amount": quote.out_amount,
                "fee_amount": quote.fee_amount,
                "fee_mint": quote.fee_mint.to_string(),
            }),
        }),
        Err(e) => Json(WebJsonResponse {
            status: Status::Error,
            message: format!("Failed to get quote: {}", e),
            data: json!({}),
        }),
    };

    result
}

async fn simulate_swap(
    State(_ctx): State<Arc<AppContext>>,
    Json(body): Json<serde_json::Value>,
) -> Json<WebJsonResponse> {
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

    Json(WebJsonResponse {
        status: Status::Success,
        message: result.to_string(),
        data: result,
    })
}
