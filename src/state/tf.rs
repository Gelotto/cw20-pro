use crate::{msg::tf::NewDenomMetadata, tf::tokenfactory::TokenFactoryType};
use cosmwasm_std::{Addr, Uint256, Uint64};
use cw_storage_plus::Item;

// Core TokenFactory state
// -----------------------------------------------------------------------------
/// Platform-specific bindings for the targeted tokenfactory implementation
pub const TF_FACTORY: Item<TokenFactoryType> = Item::new("tf_factory");
/// Full denom, like 'factory/{contractAddr}/{subdenom}
pub const TF_FULL_DENOM: Item<String> = Item::new("tf_full_denom");
/// Mint reply ID counter
pub const TF_REPLY_ID_COUNTER: Item<Uint64> = Item::new("tf_reply_id_counter");
/// Denom metadata set through this contract
pub const TF_METADATA: Item<NewDenomMetadata> = Item::new("tf_metadata");
/// Total amount of token minted through this contract
pub const TF_AMOUNT_MINTED: Item<Uint256> = Item::new("tf_amount_minted");
/// Total amount of token burned through this contract
pub const TF_AMOUNT_BURNED: Item<Uint256> = Item::new("tf_amount_burned");

// State used only by the TokenFactory's  MintInitialBalances routine
// -----------------------------------------------------------------------------
/// Flag to indicate that we're done initializing tf token from this cw20
pub const TF_INITIAL_BALANCES_CURSOR: Item<Addr> = Item::new("tf_initial_balances_cursor");
/// Number of balances that have been airdropped so far
pub const TF_N_BALANCES_INITIALIZED: Item<Uint64> = Item::new("tf_n_balances_initialized");
