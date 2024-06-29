use crate::{
    error::ContractError,
    math::{add_u128, add_u64, sub_u128, sub_u64},
    state::{N_BALANCES, RANKED_BALANCES},
};
use cosmwasm_std::{Addr, Api, Storage, Uint128};
use cw20_base::state::BALANCES;

/// Custom business logic that fires before the cw20-base send and transfer
/// functions.
pub fn before_transfer(
    store: &mut dyn Storage,
    api: &dyn Api,
    sender: &Addr,
    recipient: &String,
    delta: Uint128,
) -> Result<(), ContractError> {
    let recipient = api.addr_validate(&recipient)?;
    adjust_ranked_balances(store, sender, &recipient, delta)?;
    Ok(())
}

fn adjust_ranked_balances(
    store: &mut dyn Storage,
    sender: &Addr,
    recipient: &Addr,
    delta: Uint128,
) -> Result<(), ContractError> {
    // Adjust senders's entry in ranke balances map
    {
        let prev_balance = BALANCES.load(store, &sender).unwrap_or_default();
        let next_balance = sub_u128(prev_balance, delta)?;

        RANKED_BALANCES.remove(store, (prev_balance.u128(), &sender));
        if !next_balance.is_zero() {
            RANKED_BALANCES.save(store, (next_balance.u128(), &sender), &0)?;
        } else {
            N_BALANCES.update(store, |n| sub_u64(n, 1u64))?;
        }
    }

    // Adjust recipient's entry in ranke balances map
    {
        let prev_balance = BALANCES.load(store, &recipient).unwrap_or_default();
        let next_balance = add_u128(prev_balance, delta)?;

        RANKED_BALANCES.remove(store, (prev_balance.u128(), &recipient));
        if !next_balance.is_zero() {
            RANKED_BALANCES.save(store, (next_balance.u128(), &recipient), &0)?;
            if prev_balance.is_zero() {
                N_BALANCES.update(store, |n| add_u64(n, 1u64))?;
            }
        }
    }

    Ok(())
}
