use crate::{
    error::ContractError,
    math::{add_u128, add_u64},
    msg::BalanceCopyMode,
    state::{BALANCE_COPY_CURSORS, N_BALANCES, ORDERED_BALANCES},
};
use cosmwasm_std::{attr, Addr, DepsMut, Response, Uint128};
use cw20::{AllAccountsResponse, BalanceResponse, Cw20QueryMsg};
use cw20_base::state::BALANCES;

const ALL_BALANCES_QUERY_LIMIT: u32 = 30;
const NUM_BALANCES_QUERY_PER_EXECUTION: u32 = 2;
const LIMIT: u64 = 50;

pub fn exec_copy_cw20_balances(
    deps: DepsMut,
    other_cw20_addr: Addr,
    copy_mode: BalanceCopyMode,
) -> Result<Response, ContractError> {
    let mut cursor: Option<String> = BALANCE_COPY_CURSORS.may_load(deps.storage, &other_cw20_addr)?;

    // Fail if we're already in the middle of copying some other CW20. Just one
    // at a time, baby...
    if !BALANCE_COPY_CURSORS.is_empty(deps.storage) && !BALANCE_COPY_CURSORS.has(deps.storage, &other_cw20_addr) {
        return Err(ContractError::Unauthorized {
            reason: format!(
                "copy already in progress for {}",
                BALANCE_COPY_CURSORS.first(deps.storage)?.unwrap().1.to_string()
            ),
        });
    }

    let mut t = 0;

    for _ in 0..NUM_BALANCES_QUERY_PER_EXECUTION {
        // fetch next batch of cw20 account address
        let AllAccountsResponse { accounts } = deps.querier.query_wasm_smart(
            deps.api.addr_validate(other_cw20_addr.as_str())?,
            &Cw20QueryMsg::AllAccounts {
                start_after: cursor,
                limit: Some(ALL_BALANCES_QUERY_LIMIT),
            },
        )?;

        // iterate over the cw20's balances in batches of 30, since this is the
        // max limit on its all_accounts query :(
        for address in accounts.iter() {
            let BalanceResponse { balance } = deps.querier.query_wasm_smart(
                other_cw20_addr.to_owned(),
                &Cw20QueryMsg::Balance {
                    address: address.to_owned(),
                },
            )?;

            // Overwrite or increment existing balances based on copy mode
            let address = Addr::unchecked(address);
            let mut old_balance = Uint128::zero();
            let new_balance = BALANCES.update(deps.storage, &address, |maybe_amount| -> Result<_, ContractError> {
                old_balance = maybe_amount.unwrap_or_default();
                Ok(match copy_mode {
                    BalanceCopyMode::Replace => balance,
                    BalanceCopyMode::Increment => add_u128(old_balance, balance)?,
                })
            })?;

            // Update its ordered balances entry
            ORDERED_BALANCES.remove(deps.storage, (old_balance.u128(), &address));
            if !new_balance.is_zero() {
                ORDERED_BALANCES.save(deps.storage, (new_balance.u128(), &address), &0)?;
                if old_balance.is_zero() {
                    N_BALANCES.update(deps.storage, |n| add_u64(n, 1u64))?;
                }
            }

            t += 1;
            if t == LIMIT {
                break;
            }
        }

        // update cursor
        if accounts.len() == ALL_BALANCES_QUERY_LIMIT as usize {
            cursor = accounts.last().and_then(|a| Some(a.to_owned()));
        } else {
            cursor = None;
            break;
        }

        if t == LIMIT {
            break;
        }
    }
    let mut attrs = vec![attr("action", "copy_balances")];

    // Persist final cursor
    if let Some(cursor) = &cursor {
        BALANCE_COPY_CURSORS.save(deps.storage, &other_cw20_addr, cursor)?;
        attrs.push(attr("done", "false"));
    } else {
        BALANCE_COPY_CURSORS.remove(deps.storage, &other_cw20_addr);
        attrs.push(attr("done", "true"));
    }

    Ok(Response::new().add_attributes(attrs))
}
