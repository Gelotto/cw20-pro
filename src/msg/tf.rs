use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint256};

use crate::tf::{
    cosmos::common::{DenomUnit, Metadata},
    tokenfactory::TokenFactoryType,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub manager: Option<Addr>,
    pub factory: Option<TokenFactoryType>,
    pub initial_balances: Option<Vec<MintParams>>,
    pub metadata: NewDenomMetadata,
}

#[cw_serde]
pub struct NewDenomMetadata {
    pub symbol: String,
    pub decimals: u32,
    pub name: String,
    pub description: Option<String>,
    pub uri: Option<String>,
}

impl NewDenomMetadata {
    pub fn to_token_factory_metadata(
        &self,
        factory: TokenFactoryType,
        full_denom: &String,
    ) -> Metadata {
        let denom_units = match factory {
            _ => vec![
                DenomUnit {
                    aliases: vec![],
                    denom: full_denom.to_owned(),
                    exponent: 0,
                },
                DenomUnit {
                    aliases: vec![],
                    denom: self.symbol.to_owned(),
                    exponent: self.decimals,
                },
            ],
        };
        Metadata {
            symbol: self.symbol.to_owned(),
            display: self.symbol.to_owned(),
            name: self.name.to_owned(),
            base: denom_units[0].denom.to_owned(),
            description: self.description.to_owned().unwrap_or_default(),
            uri: self.uri.to_owned().unwrap_or_default(),
            denom_units,
        }
    }

    pub fn build_micro_denom(full_denom: &String) -> String {
        let mut parts = full_denom.split('/').map(|s| s.to_owned()).collect::<Vec<String>>();

        if !parts.is_empty() {
            // Prepend 'u' to the last element
            let micro_subdenom = format!("u{}", parts[parts.len() - 1]);
            parts.pop();
            parts.push(micro_subdenom);
        }
        // Join the parts back into a single string
        parts.join("/")
    }
}

#[cw_serde]
pub struct MintParams {
    pub address: Addr,
    pub amount: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    Mint { recipient: Addr, amount: Uint128 },
    MintMultiple { recipients: Vec<(Addr, Uint128)> },
    Burn { amount: Uint128 },
    SetManager { address: Addr },
    SetDenomMetadata { metadata: NewDenomMetadata },
    SetDenomAdmin { address: Addr },
    RemoveDenomAdmin {},
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Info {},
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct ContractStats {
    pub amount_burned: Uint256,
    pub amount_minted: Uint256,
}

#[cw_serde]
pub struct InfoResponse {
    pub factory: TokenFactoryType,
    pub denom: String,
    pub minimal_denom: String,
    pub metadata: NewDenomMetadata,
    pub stats: ContractStats,
}
