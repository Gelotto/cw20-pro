pub mod tf;

use std::collections::HashMap;

use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdError, Uint128, Uint64};
use cw20::{Cw20Coin, EmbeddedLogo, Logo, LogoInfo, MarketingInfoResponse};
use cw20_base::{
    contract::create_accounts,
    state::{MinterData, TokenInfo, LOGO, MARKETING_INFO, TOKEN_INFO},
};
use cw_storage_plus::{Item, Map};
use tf::TF_N_BALANCES_INITIALIZED;

use crate::{error::ContractError, math::add_u128};

const LOGO_SIZE_CAP: usize = 5 * 1024;

pub const OPERATOR_ADDR: Item<Addr> = Item::new("operator_addr");
pub const ORDERED_BALANCES: Map<(u128, &Addr), u8> = Map::new("ordered_balances");
pub const N_BALANCES: Item<Uint64> = Item::new("n_balances");
pub const GLOBAL_BALANCE_FREEZE: Item<bool> = Item::new("global_balance_freeze");
pub const FROZEN_ACCOUNTS: Map<&Addr, bool> = Map::new("frozen_accounts");
pub const BALANCE_COPY_CURSORS: Map<&Addr, String> = Map::new("balance_copy_cursors");

/// Top-level initialization of contract state
pub fn init(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: cw20_base::msg::InstantiateMsg,
) -> Result<Response, ContractError> {
    OPERATOR_ADDR.save(deps.storage, &info.sender)?;
    GLOBAL_BALANCE_FREEZE.save(deps.storage, &false)?;
    TF_N_BALANCES_INITIALIZED.save(deps.storage, &Uint64::zero())?;

    let mut non_zero_initial_balances: Vec<Cw20Coin> = Vec::with_capacity(msg.initial_balances.len());
    if !msg.initial_balances.is_empty() {
        let mut initial_balances_hmap: HashMap<String, Uint128> = HashMap::with_capacity(msg.initial_balances.len());
        for b in msg.initial_balances.iter() {
            initial_balances_hmap.insert(
                b.address.to_owned(),
                add_u128(
                    *initial_balances_hmap.get(&b.address).unwrap_or(&Uint128::zero()),
                    b.amount,
                )?,
            );
        }
        for (address, amount) in initial_balances_hmap.iter() {
            // Use unchecked because the cw20 base create_accounts already
            // validates the addrs.
            if !amount.is_zero() {
                let address = Addr::unchecked(address);
                ORDERED_BALANCES.save(deps.storage, (amount.u128(), &address), &0)?;
                non_zero_initial_balances.push(Cw20Coin {
                    address: address.to_string(),
                    amount: *amount,
                });
            }
        }
    }

    N_BALANCES.save(deps.storage, &Uint64::from(non_zero_initial_balances.len() as u64))?;

    // CW20-base instantiation
    //--------------------------------------
    // check valid token info
    msg.validate()?;

    // create initial accounts
    let mut deps = deps;
    let total_supply = create_accounts(&mut deps, &non_zero_initial_balances)?;

    if let Some(limit) = msg.get_cap() {
        if total_supply > limit {
            return Err(StdError::generic_err("Initial supply greater than cap").into());
        }
    }

    let mint = match msg.mint {
        Some(m) => Some(MinterData {
            minter: deps.api.addr_validate(&m.minter)?,
            cap: m.cap,
        }),
        None => None,
    };

    // store token info
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply,
        mint,
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    if let Some(marketing) = msg.marketing {
        let logo = if let Some(logo) = marketing.logo {
            verify_logo(&logo)?;
            LOGO.save(deps.storage, &logo)?;

            match logo {
                Logo::Url(url) => Some(LogoInfo::Url(url)),
                Logo::Embedded(_) => Some(LogoInfo::Embedded),
            }
        } else {
            None
        };

        let data = MarketingInfoResponse {
            project: marketing.project,
            description: marketing.description,
            marketing: marketing
                .marketing
                .map(|addr| deps.api.addr_validate(&addr))
                .transpose()?,
            logo,
        };
        MARKETING_INFO.save(deps.storage, &data)?;
    }

    Ok(Response::new().add_attribute("action", "instantiate"))
}

/// Checks if data starts with XML preamble
fn verify_xml_preamble(data: &[u8]) -> Result<(), ContractError> {
    // The easiest way to perform this check would be just match on regex, however regex
    // compilation is heavy and probably not worth it.

    let preamble = data
        .split_inclusive(|c| *c == b'>')
        .next()
        .ok_or(ContractError::InvalidXmlPreamble {})?;

    const PREFIX: &[u8] = b"<?xml ";
    const POSTFIX: &[u8] = b"?>";

    if !(preamble.starts_with(PREFIX) && preamble.ends_with(POSTFIX)) {
        Err(ContractError::InvalidXmlPreamble {})
    } else {
        Ok(())
    }

    // Additionally attributes format could be validated as they are well defined, as well as
    // comments presence inside of preable, but it is probably not worth it.
}

/// Validates XML logo
fn verify_xml_logo(logo: &[u8]) -> Result<(), ContractError> {
    verify_xml_preamble(logo)?;

    if logo.len() > LOGO_SIZE_CAP {
        Err(ContractError::LogoTooBig {})
    } else {
        Ok(())
    }
}

/// Validates png logo
fn verify_png_logo(logo: &[u8]) -> Result<(), ContractError> {
    // PNG header format:
    // 0x89 - magic byte, out of ASCII table to fail on 7-bit systems
    // "PNG" ascii representation
    // [0x0d, 0x0a] - dos style line ending
    // 0x1a - dos control character, stop displaying rest of the file
    // 0x0a - unix style line ending
    const HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    if logo.len() > LOGO_SIZE_CAP {
        Err(ContractError::LogoTooBig {})
    } else if !logo.starts_with(&HEADER) {
        Err(ContractError::InvalidPngHeader {})
    } else {
        Ok(())
    }
}

/// Checks if passed logo is correct, and if not, returns an error
fn verify_logo(logo: &Logo) -> Result<(), ContractError> {
    match logo {
        Logo::Embedded(EmbeddedLogo::Svg(logo)) => verify_xml_logo(logo),
        Logo::Embedded(EmbeddedLogo::Png(logo)) => verify_png_logo(logo),
        Logo::Url(_) => Ok(()), // Any reasonable url validation would be regex based, probably not worth it
    }
}
