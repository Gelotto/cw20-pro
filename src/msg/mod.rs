pub mod tf;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Uint128};
use cw20::{Expiration, Logo};

#[cw_serde]
pub enum OperatorExecuteMsg {
    RemoveOperator {},
    SetOperator {
        address: Addr,
    },
    FreezeBalances {
        addresses: Option<Vec<Addr>>,
    },
    UnfreezeBalances {
        addresses: Option<Vec<Addr>>,
    },
    CopyBalances {
        cw20_address: Addr,
        mode: BalanceCopyMode,
    },
    UpdateBalanceChangeListeners {
        add: Option<Vec<Addr>>,
        remove: Option<Vec<Addr>>,
    },
}

#[cw_serde]
pub enum TokenFactoryExecuteMsg {
    /// Copy the current CW20's balances to the tokenfactory denom. This may
    /// require multiple transactions, depending on the number of accounts. A
    /// true/false "done" attribute is returned in tx events.
    DeriveBalances {
        limit: Option<u16>,
    },
    /// Create a new tokenfactory token AKA bank denom, initialized to the same
    /// parameters used by the CW20 -- name, decimals, etc.
    DeriveDenom {},
    /// Mint and send tokens to a list of recipients. Expects a vec of
    /// (recipient, amount) pairs.
    Mint {
        recipients: Vec<(Addr, Uint128)>,
    },
    Burn {
        amount: Uint128,
    },
    SetMetadata {
        metadata: tf::NewDenomMetadata,
    },
    SetAdmin {
        address: Addr,
    },
    RemoveAdmin {},
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Tokenfactory-related functions
    Pro(OperatorExecuteMsg),
    /// Tokenfactory-related functions
    TokenFactory(TokenFactoryExecuteMsg),

    /// Implements CW20. Transfer is a base message to move tokens to another
    /// account without triggering actions
    Transfer { recipient: String, amount: Uint128 },
    /// Implements CW20. Only with the "mintable" extension. If authorized,
    /// creates amount new tokens and adds to the recipient balance.
    Mint { recipient: String, amount: Uint128 },
    /// Only with the "mintable" extension. The current minter may set
    /// a new minter. Setting the minter to None will remove the
    /// token's minter forever.
    UpdateMinter { new_minter: Option<String> },
    /// Implements CW20. Burn is a base message to destroy tokens forever
    Burn { amount: Uint128 },
    /// Implements CW20.  Send is a base message to transfer tokens to a
    /// contract and trigger an action on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Implements CW20 "approval" extension. Allows spender to access an
    /// additional amount tokens from the owner's (env.sender) account. If
    /// expires is Some(), overwrites current allowance expiration with this
    /// one.
    IncreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Implements CW20 "approval" extension. Lowers the spender's access of
    /// tokens from the owner's (env.sender) account by amount. If expires is
    /// Some(), overwrites current allowance expiration with this one.
    DecreaseAllowance {
        spender: String,
        amount: Uint128,
        expires: Option<Expiration>,
    },
    /// Implements CW20 "approval" extension. Transfers amount tokens from owner
    /// -> recipient if `env.sender` has sufficient pre-approval.
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    /// Implements CW20 "approval" extension. Sends amount tokens from owner ->
    /// contract if `env.sender` has sufficient pre-approval.
    SendFrom {
        owner: String,
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Implements CW20 "approval" extension. Destroys tokens forever
    BurnFrom { owner: String, amount: Uint128 },
    /// Only with the "marketing" extension. If authorized, updates marketing
    /// metadata.  Setting None/null for any of these will leave it unchanged.
    /// Setting Some("") will clear this field on the contract storage
    UpateMarketing {
        project: Option<String>,
        description: Option<String>,
        marketing: Option<String>,
    },
    /// CW20 update logo function
    UploadLogo(Logo),
}

#[cw_serde]
pub enum ProBalanceQueryMsg {
    All {
        limit: Option<u16>,
        desc: Option<bool>,
        cursor: Option<(Uint128, Addr)>,
    },
    ByAddress {
        addresses: Vec<Addr>,
    },
}

#[cw_serde]
pub enum BalanceCopyMode {
    Replace,
    Increment,
}

#[cw_serde]
pub enum ProQueryMsg {
    Balances(ProBalanceQueryMsg),
}

#[cw_serde]
pub enum QueryMsg {
    Pro(ProQueryMsg),

    /// Implements CW20. Returns current balance of the given address, else 0.
    Balance {
        address: String,
    },
    /// Implements CW20. Returns metadata on the contract - name, decimals,
    /// supply, etc.
    TokenInfo {},
    /// Implements CW20 "allowance" extension. Returns how much spender can use
    /// from owner account, 0 if unset.
    Allowance {
        owner: String,
        spender: String,
    },
    /// Only with "enumerable" extension
    /// Returns all accounts that have balances. Supports pagination.
    /// Return type: AllAccountsResponse.
    AllAccounts {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Only with "mintable" extension.
    /// Returns who can mint and the hard cap on maximum tokens after minting.
    /// Return type: MinterResponse.
    Minter {},
    /// Only with "marketing" extension
    /// Returns more metadata on the contract to display in the client:
    /// - description, logo, project url, etc.
    /// Return type: MarketingInfoResponse.
    MarketingInfo {},
    /// Only with "marketing" extension
    /// Downloads the embedded logo data (if stored on chain). Errors if no logo data stored for
    /// this contract.
    /// Return type: DownloadLogoResponse.
    DownloadLogo {},
    /// Only with "enumerable" extension (and "allowances")
    /// Returns all allowances this owner has approved. Supports pagination.
    /// Return type: AllAllowancesResponse.
    AllAllowances {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    AllSpenderAllowances {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
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

#[cw_serde]
pub enum BalanceChangeEvent {
    Transfer {
        initiator: Addr,
        recipient: Addr,
        amount: Uint128,
    },
    Burn {
        initiator: Addr,
        amount: Uint128,
    },
    Mint {
        initiator: Addr,
        recipient: Addr,
        amount: Uint128,
    },
}

#[cw_serde]
pub enum BalanceChangeListenerInterface {
    OnBalanceChange { event: BalanceChangeEvent },
}
