use crate::{
    error::ContractError,
    math::{sub_u128, sub_u64},
    state::{N_BALANCES, RANKED_BALANCES},
};
use cosmwasm_std::{Api, Storage, Uint128};
use cw20_base::state::BALANCES;

/// Custom business logic that executes BEFORE the cw20 base mint function
pub fn before_burn(
    store: &mut dyn Storage,
    api: &dyn Api,
    burner: &String,
    delta: Uint128,
) -> Result<(), ContractError> {
    let burner = api.addr_validate(&burner)?;
    let prev_balance: Uint128 = BALANCES.load(store, &burner).unwrap_or_default();
    let curr_balance = sub_u128(prev_balance, delta)?;

    // Update the account's entry in the RANKED_BALANCES map and decrement the
    // aggregate balance counter if necessary.
    RANKED_BALANCES.remove(store, (prev_balance.u128(), &burner));
    if !curr_balance.is_zero() {
        RANKED_BALANCES.save(store, (curr_balance.u128(), &burner), &0)?;
        N_BALANCES.update(store, |n| sub_u64(n, 1u64))?;
    }

    Ok(())
}
