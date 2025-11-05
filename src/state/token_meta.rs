use std::sync::Arc;

use anyhow::Result;
use mpl_token_metadata::accounts::Metadata;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{account::Account, program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Mint;
use spl_token_2022_interface::{
    extension::{metadata_pointer::MetadataPointer, BaseStateWithExtensions, StateWithExtensions},
    state::Mint as Mint2022,
};

use spl_token_metadata_interface::state::TokenMetadata;
use tracing::info;

#[derive(Clone)]
pub struct TokenMeta {
    pub mint: Pubkey,
    pub symbol: String,
    pub decimals: u8,
}

impl Default for TokenMeta {
    fn default() -> Self {
        TokenMeta {
            mint: Pubkey::default(),
            symbol: "UNKNOWN".to_string(),
            decimals: 0,
        }
    }
}

impl TokenMeta {
    pub fn fetch(client: Arc<RpcClient>, mint_key: Pubkey) -> Result<TokenMeta> {
        let token_account = client.get_account(&mint_key)?;
        match token_account.owner {
            spl_token::ID => {
                let mint_account = spl_token::state::Mint::unpack(&token_account.data)?;
                let token_meta = Self::get_spl_token_metadata(client, &mint_key)?;
                return Ok(TokenMeta {
                    mint: mint_key,
                    decimals: mint_account.decimals,
                    symbol: token_meta.symbol,
                    ..Default::default()
                });
            }
            spl_token_2022::ID => {
                let mint_state = StateWithExtensions::<Mint2022>::unpack(&token_account.data)?;
                // Get all extension types enabled on this mint
                // let extension_types = mint_state.get_extension_types()?;
                // info!("\nExtensions enabled: {:?}", extension_types);

                // Deserialize the MetadataPointer extension data
                // let metadata_pointer = mint_state.get_extension::<MetadataPointer>()?;
                // info!("\n{:#?}", metadata_pointer);

                // Deserialize the TokenMetadata extension data (variable-length)
                let token_metadata = mint_state.get_variable_len_extension::<TokenMetadata>()?;
                info!("\n{:#?}", token_metadata);

                // info!("Fetched token metadata: {:?}", token_meta);
                return Ok(TokenMeta {
                    mint: mint_key,
                    decimals: mint_state.base.decimals,
                    symbol: token_metadata.symbol.clone(),
                    ..Default::default()
                });
            }
            _ => {
                return Err(anyhow::anyhow!("Account is not owned by SPL Token program"));
            }
        }
    }

    fn get_spl_token_metadata(client: Arc<RpcClient>, mint_key: &Pubkey) -> Result<Metadata> {
        let (metadata_pda, _) = Pubkey::find_program_address(
            &[
                b"metadata",
                mpl_token_metadata::ID.as_ref(),
                mint_key.as_ref(),
            ],
            &mpl_token_metadata::ID,
        );

        let account = client.get_account_data(&metadata_pda)?;

        let metadata = Metadata::safe_deserialize(&mut &account[..])?;
        Ok(Metadata {
            name: metadata.name,
            symbol: metadata.symbol,
            uri: metadata.uri,
            key: metadata.key,
            update_authority: metadata.update_authority,
            mint: metadata.mint,
            seller_fee_basis_points: metadata.seller_fee_basis_points,
            creators: metadata.creators,
            primary_sale_happened: metadata.primary_sale_happened,
            is_mutable: metadata.is_mutable,
            edition_nonce: metadata.edition_nonce,
            token_standard: metadata.token_standard,
            collection: metadata.collection,
            uses: metadata.uses,
            collection_details: metadata.collection_details,
            programmable_config: metadata.programmable_config,
        })
    }
}
