use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Uint128};
use cw20::Expiration;

pub const MIN_LOCKUP_SECONDS: u64 = 60 * 60; // one hour

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum TokenFactoryExecuteMsg {
    Airdrop {},
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Tokenfactory-related functions
    TokenFactory(TokenFactoryExecuteMsg),

    /// Implements CW20. Transfer is a base message to move tokens to another account without triggering actions
    Transfer { recipient: String, amount: Uint128 },
    /// Implements CW20. Burn is a base message to destroy tokens forever
    Burn { amount: Uint128 },
    /// Implements CW20.  Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Implements CW20 "approval" extension. Allows spender to access an additional amount tokens
    /// from the owner's (env.sender) account. If expires is Some(), overwrites current allowance
    /// expiration with this one.
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Implements CW20 "approval" extension. Lowers the spender's access of tokens
    /// from the owner's (env.sender) account by amount. If expires is Some(), overwrites current
    /// allowance expiration with this one.
    DecreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Implements CW20 "approval" extension. Transfers amount tokens from owner -> recipient
    /// if `env.sender` has sufficient pre-approval.
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    /// Implements CW20 "approval" extension. Sends amount tokens from owner -> contract
    /// if `env.sender` has sufficient pre-approval.
    SendFrom {
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Implements CW20 "approval" extension. Destroys tokens forever
    BurnFrom { owner: String, amount: Uint128 },
}

#[cw_serde]
pub enum ProQueryMsg {
    Balances {
        limit: Option<u16>,
        cursor: Option<(Uint128, Addr)>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Pro(ProQueryMsg),

    /// Implements CW20. Returns the current balance of the given address, 0 if unset.
    Balance {
        address: String,
    },
    /// Implements CW20. Returns metadata on the contract - name, decimals, supply, etc.
    TokenInfo {},
    /// Implements CW20 "allowance" extension.
    /// Returns how much spender can use from owner account, 0 if unset.
    Allowance {
        owner: String,
        spender: String,
    },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct AccountBalance {
    pub address: Addr,
    pub amount: Uint128,
}

#[cw_serde]
pub struct BalancesResponse {
    pub balances: Vec<AccountBalance>,
    pub cursor: Option<(Uint128, Addr)>,
}
