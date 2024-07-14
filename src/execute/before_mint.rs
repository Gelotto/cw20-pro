use crate::{
    checks::ensure_accounts_not_frozen,
    error::ContractError,
    math::{add_u128, add_u64},
    state::{N_BALANCES, ORDERED_BALANCES},
};
use cosmwasm_std::{Addr, Api, Storage, Uint128};
use cw20_base::state::BALANCES;

/// Custom business logic that executes BEFORE the cw20 base mint function
pub fn before_mint(
    store: &mut dyn Storage,
    api: &dyn Api,
    recipient: &String,
    delta: Uint128,
) -> Result<(), ContractError> {
    let recipient = api.addr_validate(&recipient)?;
    ensure_accounts_not_frozen(store, None, Some(recipient.to_owned()))?;
    update_ordered_balance(store, &recipient, delta)?;
    Ok(())
}

/// Update the account's entry in the RANKED_BALANCES map and increment the
/// aggregate balance counter if necessary.
pub fn update_ordered_balance(
    store: &mut dyn Storage,
    address: &Addr,
    delta: Uint128,
) -> Result<(), ContractError> {
    let prev_balance = BALANCES.load(store, &address).unwrap_or_default();
    let next_balance = add_u128(prev_balance, delta)?;

    ORDERED_BALANCES.remove(store, (prev_balance.u128(), &address));
    if !next_balance.is_zero() {
        ORDERED_BALANCES.save(store, (next_balance.u128(), &address), &0)?;
        if prev_balance.is_zero() {
            N_BALANCES.update(store, |n| add_u64(n, 1u64))?;
        }
    }

    Ok(())
}
