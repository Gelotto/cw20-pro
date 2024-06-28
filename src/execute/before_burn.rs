use crate::{error::ContractError, math::sub_u128, state::storage::RANKED_BALANCES};
use cosmwasm_std::{Api, DepsMut, Storage, Uint128};
use cw20_base::state::BALANCES;

/// Update the account's position in the data structure keeping track of account
/// addresses ordered by balance, which we use to scan for winners in order of
/// largest to smallest balance during the draw process.
pub fn before_burn(
    store: &mut dyn Storage,
    api: &dyn Api,
    burner: &String,
    delta: Uint128,
) -> Result<(), ContractError> {
    let burner = api.addr_validate(&burner)?;
    let prev_balance: Uint128 = BALANCES.load(store, &burner).unwrap_or_default();
    let curr_balance = sub_u128(prev_balance, delta)?;

    RANKED_BALANCES.remove(store, (prev_balance.u128(), &burner));
    if !curr_balance.is_zero() {
        RANKED_BALANCES.save(store, (curr_balance.u128(), &burner), &0)?;
    }

    Ok(())
}
