use crate::{
    checks::ensure_accounts_not_frozen,
    error::ContractError,
    math::{sub_u128, sub_u64},
    msg::BalanceChangeEvent,
    state::{N_BALANCES, ORDERED_BALANCES},
};
use cosmwasm_std::{Addr, Api, Storage, SubMsg, Uint128};
use cw20_base::state::BALANCES;

use super::before_transfer::notify_balance_change_listeners;

/// Custom business logic that executes BEFORE the cw20 base burn function
pub fn before_burn(
    store: &mut dyn Storage,
    api: &dyn Api,
    burner: &String,
    delta: Uint128,
) -> Result<Vec<SubMsg>, ContractError> {
    let burner = api.addr_validate(&burner)?;

    ensure_accounts_not_frozen(store, Some(burner.to_owned()), None)?;
    update_ordered_balance(store, &burner, delta)?;

    let submsgs = notify_balance_change_listeners(
        store,
        &BalanceChangeEvent::Burn {
            initiator: burner.to_owned(),
            amount: delta,
        },
    )?;

    Ok(submsgs)
}

/// Update the account's entry in the RANKED_BALANCES map and decrement the
/// aggregate balance counter if necessary.
fn update_ordered_balance(
    store: &mut dyn Storage,
    burner: &Addr,
    delta: Uint128,
) -> Result<(), ContractError> {
    let prev_balance: Uint128 = BALANCES.load(store, burner).unwrap_or_default();
    let curr_balance = sub_u128(prev_balance, delta)?;

    ORDERED_BALANCES.remove(store, (prev_balance.u128(), burner));
    if !curr_balance.is_zero() {
        ORDERED_BALANCES.save(store, (curr_balance.u128(), burner), &0)?;
        N_BALANCES.update(store, |n| sub_u64(n, 1u64))?;
    }

    Ok(())
}
