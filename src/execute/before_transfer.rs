use crate::{
    checks::{ensure_accounts_not_frozen, ensure_not_self_transfer},
    error::ContractError,
    math::{add_u128, add_u64, sub_u128, sub_u64},
    msg::{BalanceChangeEvent, BalanceChangeListenerInterface},
    state::{BALANCE_CHANGE_LISTENERS, N_BALANCES, ORDERED_BALANCES},
};
use cosmwasm_std::{to_json_binary, Addr, Api, Storage, SubMsg, Uint128, WasmMsg};
use cw20_base::state::BALANCES;

/// Custom business logic that fires before the cw20-base send and transfer
/// functions.
pub fn before_transfer(
    store: &mut dyn Storage,
    api: &dyn Api,
    sender: &Addr,
    recipient: &String,
    delta: Uint128,
) -> Result<Vec<SubMsg>, ContractError> {
    let recipient = api.addr_validate(&recipient)?;

    ensure_not_self_transfer(sender, &recipient)?;
    ensure_accounts_not_frozen(store, Some(sender.to_owned()), Some(recipient.to_owned()))?;
    update_ordered_balances(store, sender, &recipient, delta)?;

    let submsgs = notify_balance_change_listeners(
        store,
        &BalanceChangeEvent::Transfer {
            initiator: sender.to_owned(),
            recipient: recipient.to_owned(),
            amount: delta,
        },
    )?;

    Ok(submsgs)
}

/// Update index for paginating accounts by balance amounts.
pub fn update_ordered_balances(
    store: &mut dyn Storage,
    sender: &Addr,
    recipient: &Addr,
    delta: Uint128,
) -> Result<(), ContractError> {
    // Adjust senders's entry in ordered balances map
    {
        let prev_balance = BALANCES.load(store, &sender).unwrap_or_default();
        let next_balance = sub_u128(prev_balance, delta)?;

        ORDERED_BALANCES.remove(store, (prev_balance.u128(), &sender));
        if !next_balance.is_zero() {
            ORDERED_BALANCES.save(store, (next_balance.u128(), &sender), &0)?;
        } else {
            N_BALANCES.update(store, |n| sub_u64(n, 1u64))?;
        }
    }

    // Adjust recipient's entry in ordered balances map
    {
        let prev_balance = BALANCES.load(store, &recipient).unwrap_or_default();
        let next_balance = add_u128(prev_balance, delta)?;

        ORDERED_BALANCES.remove(store, (prev_balance.u128(), &recipient));
        if !next_balance.is_zero() {
            ORDERED_BALANCES.save(store, (next_balance.u128(), &recipient), &0)?;
            if prev_balance.is_zero() {
                N_BALANCES.update(store, |n| add_u64(n, 1u64))?;
            }
        }
    }

    Ok(())
}

pub fn notify_balance_change_listeners(
    store: &dyn Storage,
    event: &BalanceChangeEvent,
) -> Result<Vec<SubMsg>, ContractError> {
    let mut submsgs = Vec::with_capacity(1);
    if let Some(listener_addrs) = BALANCE_CHANGE_LISTENERS.may_load(store)? {
        for contract_addr in listener_addrs.iter() {
            submsgs.push(SubMsg::new(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                funds: vec![],
                msg: to_json_binary(&BalanceChangeListenerInterface::OnBalanceChange {
                    event: event.to_owned(),
                })?,
            }))
        }
    }
    Ok(submsgs)
}
