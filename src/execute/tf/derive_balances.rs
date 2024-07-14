use std::marker::PhantomData;

use cosmwasm_std::{attr, Addr, DepsMut, Env, Event, Order, Response, Uint128};
use cw20_base::state::BALANCES;
use cw_storage_plus::Bound;

use crate::{
    error::ContractError,
    execute::tf::mint::mint_multiple,
    math::add_u64,
    state::{
        tf::{TF_INITIAL_BALANCES_CURSOR, TF_N_BALANCES_INITIALIZED},
        N_BALANCES,
    },
};

const MAX_BALANCE_BATCH_SIZE: usize = 500;
const DEFAULT_BALANCE_BATCH_SIZE: usize = 150;

pub fn exec_tf_derive_balances(
    deps: DepsMut,
    env: Env,
    limit: Option<u16>,
) -> Result<Response, ContractError> {
    let cursor_addr = TF_INITIAL_BALANCES_CURSOR
        .may_load(deps.storage)?
        .unwrap_or(Addr::unchecked("".to_owned()));

    let min_bound = deps
        .api
        .addr_validate(cursor_addr.as_str())
        .and_then(|_| Ok(Some(Bound::Exclusive((&cursor_addr, PhantomData)))))
        .unwrap_or(None);

    // Build next batch of recipient addrs & amnounts (initial balances)
    let mut recipients: Vec<(Addr, Uint128)> = Vec::with_capacity(N_BALANCES.load(deps.storage)?.u64() as usize);
    for result in BALANCES.range(deps.storage, min_bound, None, Order::Ascending).take(
        limit
            .and_then(|x| Some(x as usize))
            .unwrap_or(DEFAULT_BALANCE_BATCH_SIZE)
            .clamp(1, MAX_BALANCE_BATCH_SIZE) as usize,
    ) {
        recipients.push(result?);
    }

    let n_accounts_minted = TF_N_BALANCES_INITIALIZED.update(deps.storage, |n| -> Result<_, ContractError> {
        add_u64(n, recipients.len() as u64)
    })?;

    let n_accounts = N_BALANCES.load(deps.storage)?.u64();

    let resp = Response::new().add_attributes(vec![
        attr("action", "tf_derive_balances"),
        attr("batch_size", recipients.len().to_string()),
        attr("done", (n_accounts == n_accounts_minted.u64()).to_string()),
    ]);

    // Mint coins to airdrop recipients via submsgs
    Ok(if !recipients.is_empty() {
        TF_INITIAL_BALANCES_CURSOR.save(deps.storage, &recipients.last().unwrap().0)?;
        let (mint_amount, transfer_submsg) = mint_multiple(deps.storage, &env.contract.address, &recipients)?;
        resp.add_submessage(transfer_submsg)
            .add_event(Event::new("airdrop").add_attributes(vec![
                attr("mint_amount", mint_amount.u128().to_string()),
                attr("done", (n_accounts == n_accounts_minted.u64()).to_string()),
                attr("n_balances_processed", n_accounts_minted.u64().to_string()),
                attr("n_balances_total", n_accounts.to_string()),
            ]))
    } else {
        resp.add_event(Event::new("airdrop").add_attributes(vec![
            attr("mint_amount", "0"),
            attr("done", (n_accounts == n_accounts_minted.u64()).to_string()),
            attr("n_balances_processed", n_accounts_minted.u64().to_string()),
            attr("n_balances_total", n_accounts.to_string()),
        ]))
    })
}
