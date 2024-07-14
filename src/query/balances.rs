use std::marker::PhantomData;

use cosmwasm_std::{Addr, Deps, Order, Uint128};
use cw20_base::state::BALANCES;
use cw_storage_plus::Bound;

use crate::{
    error::ContractError,
    msg::{AccountBalance, BalancesResponse},
    state::ORDERED_BALANCES,
};

const DEFAULT_LIMIT: u16 = 50;
const MAX_LIMIT: u16 = 500;

pub fn query_paginate_balances(
    deps: Deps,
    limit: Option<u16>,
    desc: Option<bool>,
    cursor: Option<(Uint128, Addr)>,
) -> Result<BalancesResponse, ContractError> {
    // Default the limit param to something reasonable
    let cursor = cursor.unwrap_or_else(|| (Uint128::zero(), Addr::unchecked("")));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT) as usize;
    let order = if desc.unwrap_or(true) {
        Order::Descending
    } else {
        Order::Ascending
    };

    // Build starting point to begin or resume iteratation over balances map
    let (min_bound, max_bound) = if !cursor.0.is_zero() {
        match order {
            Order::Ascending => (
                Some(Bound::Exclusive(((cursor.0.u128(), &cursor.1), PhantomData))),
                None,
            ),
            Order::Descending => (
                None,
                Some(Bound::Exclusive(((cursor.0.u128(), &cursor.1), PhantomData))),
            ),
        }
    } else {
        (None, None)
    };

    // Iterate through balances in order, create AccountBalances to return
    let balances: Vec<AccountBalance> = ORDERED_BALANCES
        .keys(deps.storage, min_bound, max_bound, order)
        .take(limit)
        .map(|result| {
            let (amount, address) = result.unwrap();
            AccountBalance {
                amount: amount.into(),
                address,
            }
        })
        .collect();

    // Return a cursor to to resume iteration in a follow-up query. Only return
    // a non-null cursor if the number of elements being returned equals the
    // input limit param AND the last element in the batch isn't equal to the
    // final ranked balance in the storage map.
    let mut next_cursor: Option<(Uint128, Addr)> = None;
    if balances.len() == limit {
        if let Some(((_, final_address), _)) = ORDERED_BALANCES.last(deps.storage)? {
            let tail = balances.last().unwrap();
            if tail.address != final_address {
                next_cursor = Some((tail.amount, tail.address.clone()));
            }
        }
    }

    Ok(BalancesResponse {
        cursor: next_cursor,
        balances,
    })
}

pub fn query_balances_by_address(
    deps: Deps,
    addresses: Vec<Addr>,
) -> Result<BalancesResponse, ContractError> {
    Ok(BalancesResponse {
        cursor: None,
        balances: addresses
            .iter()
            .map(|address| AccountBalance {
                amount: BALANCES.load(deps.storage, address).unwrap_or_default(),
                address: address.to_owned(),
            })
            .collect(),
    })
}
