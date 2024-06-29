use crate::{
    error::ContractError,
    state::{FROZEN_ACCOUNTS, GLOBAL_BALANCE_FREEZE},
};
use cosmwasm_std::{attr, Addr, DepsMut, Event, Response};
use cw20_base::state::BALANCES;

pub fn exec_freeze(
    deps: DepsMut,
    addresses: Option<Vec<Addr>>,
) -> Result<Response, ContractError> {
    Ok(Response::new()
        .add_attributes(vec![attr("action", "freeze")])
        .add_events(toggle_freeze(deps, addresses, true)?))
}

pub fn exec_unfreeze(
    deps: DepsMut,
    addresses: Option<Vec<Addr>>,
) -> Result<Response, ContractError> {
    Ok(Response::new()
        .add_attributes(vec![attr("action", "unfreeze")])
        .add_events(toggle_freeze(deps, addresses, false)?))
}

/// Lock or unlock one or more specific account or, if none specified, lock or
/// unlock all, globally.
pub fn toggle_freeze(
    deps: DepsMut,
    addresses: Option<Vec<Addr>>,
    new_state: bool,
) -> Result<Vec<Event>, ContractError> {
    if let Some(addresses) = addresses {
        // Lock specific accounts
        let mut events: Vec<Event> = Vec::with_capacity(addresses.len());
        if new_state {
            // We're setting locks
            for addr in addresses.iter() {
                if BALANCES.has(deps.storage, &deps.api.addr_validate(addr.as_str())?) {
                    events.push(Event::new("lock-account").add_attribute("address", addr.to_string()));
                    FROZEN_ACCOUNTS.save(deps.storage, &addr, &true)?;
                }
            }
        } else {
            // We're "unlocking"
            for addr in addresses.iter() {
                if BALANCES.has(deps.storage, &deps.api.addr_validate(addr.as_str())?) {
                    events.push(Event::new("unlock-account").add_attribute("address", addr.to_string()));
                    FROZEN_ACCOUNTS.remove(deps.storage, &addr);
                }
            }
        }
        Ok(events)
    } else {
        // Lock or unlock all balances globally
        if new_state {
            GLOBAL_BALANCE_FREEZE.save(deps.storage, &true)?;
        } else {
            GLOBAL_BALANCE_FREEZE.remove(deps.storage);
        }
        Ok(vec![])
    }
}
