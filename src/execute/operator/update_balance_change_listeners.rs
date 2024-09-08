use std::collections::HashSet;

use crate::{error::ContractError, state::BALANCE_CHANGE_LISTENERS};
use cosmwasm_std::{attr, Addr, DepsMut, Response};

pub fn exec_update_balance_change_listeners(
    deps: DepsMut,
    add: Option<Vec<Addr>>,
    remove: Option<Vec<Addr>>,
) -> Result<Response, ContractError> {
    let mut addr_set: HashSet<Addr> = HashSet::with_capacity(8);
    let existing_addrs = BALANCE_CHANGE_LISTENERS.load(deps.storage)?;

    addr_set.extend(existing_addrs);

    for addr in add.unwrap_or_default() {
        addr_set.insert(deps.api.addr_validate(&addr.as_str())?);
    }

    for addr in remove.unwrap_or_default() {
        addr_set.remove(&addr);
    }

    let computed_listener_addrs: Vec<Addr> = Vec::from_iter(addr_set);

    BALANCE_CHANGE_LISTENERS.save(deps.storage, &computed_listener_addrs)?;

    Ok(Response::new().add_attributes(vec![attr("action", "update_balance_change_listeners")]))
}
