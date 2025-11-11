use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{
    dlmm::DLMMClient,
    state::{MintAccount, PoolState, State, TokenMeta},
};
use anyhow::{Context, Ok, Result};
use jupiter_amm_interface::{Amm, AmmContext, ClockRef, KeyedAccount};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{clock::Clock, pubkey::Pubkey, sysvar};
use tokio::{sync::RwLock, time::Instant};
use tracing::info;

use saros_dlmm::SarosDlmm;
#[derive(Clone)]
pub struct TTLConfig {
    pub pool_ttl: Duration,
    pub token_ttl: Duration,
    pub bin_ttl: Duration,
}

#[derive(Clone)]
pub struct AppConfig {
    pub rpc_url: String,
    pub cache_ttl: TTLConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            cache_ttl: TTLConfig {
                pool_ttl: Duration::from_secs(15),     // 15 seconds
                token_ttl: Duration::from_secs(43200), // 12 hours
                bin_ttl: Duration::from_secs(15),      // 15 seconds
            },
        }
    }
}

#[derive(Clone)]
pub struct Cached<T> {
    pub value: Arc<T>,
    pub last_updated: Instant,
}

impl<T> Cached<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Arc::new(value),
            last_updated: Instant::now(),
        }
    }

    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.last_updated.elapsed() > ttl
    }
}

/// The minimal application context
#[derive(Clone)]
pub struct AppContext {
    pub config: AppConfig,
    pub rpc_client: Arc<RpcClient>,
    pub pair_accounts: Arc<RwLock<HashMap<Pubkey, Cached<KeyedAccount>>>>,
    pub pool_states: Arc<RwLock<HashMap<Pubkey, Cached<Option<PoolState>>>>>,
    pub mint_accounts: Arc<RwLock<HashMap<Pubkey, Cached<MintAccount>>>>,
    pub token_meta_cache: Arc<RwLock<HashMap<Pubkey, Cached<TokenMeta>>>>,
}

impl AppContext {
    pub fn new(config: AppConfig) -> Self {
        let rpc_client = Arc::new(RpcClient::new(config.rpc_url.clone()));
        AppContext {
            config,
            rpc_client,
            pair_accounts: Arc::new(RwLock::new(HashMap::new())),
            pool_states: Arc::new(RwLock::new(HashMap::new())),
            mint_accounts: Arc::new(RwLock::new(HashMap::new())),
            token_meta_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_or_spawn_client(&self, pool_key: Pubkey) -> Result<Arc<DLMMClient>> {
        let ttl = self.config.cache_ttl.clone();
        let mut cached_pools = self.pair_accounts.write().await;
        let mut cached_states = self.pool_states.write().await;
        let mut cached_mints = self.mint_accounts.write().await;

        let clock_data = self
            .rpc_client
            .get_account_with_commitment(&sysvar::clock::ID, self.rpc_client.commitment())?
            .value
            .context("Failed to get clock account")?;

        let clock: Clock = bincode::deserialize(&clock_data.data)
            .context("Failed to deserialize clock account data")?;

        let amm_context = AmmContext {
            clock_ref: ClockRef::try_from(clock)?,
        };

        if let Some(cached) = cached_pools.get(&pool_key) {
            if !cached.is_expired(ttl.pool_ttl) {
                info!("Using cached DLMMClient for pool: {:?}", pool_key);
                let pair_account = cached.value.as_ref().clone();
                let saros_dlmm = SarosDlmm::from_keyed_account(&pair_account, &amm_context)?;

                return Ok(Arc::new(DLMMClient {
                    saros_dlmm: Arc::new(RwLock::new(saros_dlmm)),
                }));
            } else {
                // ⚡ cleanup lazy
                cached_pools.remove(&pool_key);
                cached_states.remove(&pool_key);
            }
        }

        info!("Spawning new DLMMClient for pool: {:?}", pool_key);
        let pair_account = State::generate_keyed_account(self.rpc_client.clone(), pool_key).await?;
        cached_pools.insert(pool_key, Cached::new(pair_account.clone()));

        let state =
            State::generate_state_async(self.rpc_client.clone(), pair_account.clone()).await;

        cached_states.insert(pool_key, Cached::new(state.pool_state));

        for mint_account in state.mint_accounts.iter() {
            if cached_mints.contains_key(&mint_account.key) {
                if let Some(cached) = cached_mints.get(&mint_account.key) {
                    if !cached.is_expired(ttl.token_ttl) {
                        continue;
                    } else {
                        // ⚡ cleanup lazy
                        cached_mints.remove(&mint_account.key);
                    }
                }
            }
            cached_mints.insert(mint_account.key, Cached::new(mint_account.clone()));
        }

        let saros_dlmm = SarosDlmm::from_keyed_account(&pair_account.clone(), &amm_context)?;
        let client = Arc::new(DLMMClient {
            saros_dlmm: Arc::new(RwLock::new(saros_dlmm)),
        });

        Ok(client)
    }

    pub async fn fetch_pair_token_info(&self, dlmm_client: &SarosDlmm) -> Result<[TokenMeta; 2]> {
        let mut mint_a_state = TokenMeta::default();
        let mut mint_b_state = TokenMeta::default();

        let mut token_cache = self.token_meta_cache.write().await;
        let ttl = self.config.cache_ttl.token_ttl;

        let mint_x = dlmm_client.pair.token_mint_x;
        let mint_y = dlmm_client.pair.token_mint_y;

        if let Some(cached_mint_x) = token_cache.get(&mint_x) {
            if !cached_mint_x.is_expired(ttl) {
                info!("Using cached token meta for mint_x: {:?}", mint_x);
                mint_a_state = cached_mint_x.value.as_ref().clone();
            } else {
                // ⚡ cleanup lazy
                token_cache.remove(&mint_x);
            }
        } else {
            mint_a_state = State::generate_token_state(self.rpc_client.clone(), mint_x).await?;
            token_cache.insert(mint_x, Cached::new(mint_a_state.clone()));
        }

        if let Some(cached_mint_y) = token_cache.get(&mint_y) {
            if !cached_mint_y.is_expired(ttl) {
                info!("Using cached token meta for mint_y: {:?}", mint_y);
                mint_b_state = cached_mint_y.value.as_ref().clone();
            } else {
                // ⚡ cleanup lazy
                token_cache.remove(&mint_y);
            }
        } else {
            mint_b_state = State::generate_token_state(self.rpc_client.clone(), mint_y).await?;
            token_cache.insert(mint_y, Cached::new(mint_b_state.clone()));
        }

        Ok([mint_a_state, mint_b_state])
    }
}
