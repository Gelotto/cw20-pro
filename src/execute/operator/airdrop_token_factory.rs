use std::marker::PhantomData;

use cosmwasm_std::{attr, Addr, DepsMut, Env, Event, Order, Response, Uint128};
use cw20_base::state::BALANCES;
use cw_storage_plus::Bound;

use crate::{
    error::ContractError,
    execute::tf::mint::mint_multiple,
    state::{tf::TF_AIRDROP_BALANCES_CURSOR, N_BALANCES},
};

const MAX_BALANCE_BATCH_SIZE: usize = 500;

pub fn exec_token_factory_airdrop(
    deps: DepsMut,
    env: Env,
    limit: Option<u16>,
) -> Result<Response, ContractError> {
    let cursor_addr = TF_AIRDROP_BALANCES_CURSOR
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
            .unwrap_or(MAX_BALANCE_BATCH_SIZE)
            .clamp(1, MAX_BALANCE_BATCH_SIZE) as usize,
    ) {
        recipients.push(result?);
    }

    let resp = Response::new().add_attributes(vec![attr("action", "tf_airdrop_cw20_balances")]);

    // Mint coins to airdrop recipients
    Ok(if !recipients.is_empty() {
        TF_AIRDROP_BALANCES_CURSOR.save(deps.storage, &recipients.last().unwrap().0)?;
        let (mint_amount, transfer_submsg) = mint_multiple(deps.storage, &env.contract.address, &recipients)?;
        resp.add_submessage(transfer_submsg)
            .add_event(Event::new("airdrop").add_attributes(vec![
                attr("mint_amount", mint_amount.u128().to_string()),
                attr("n_recipients", recipients.len().to_string()),
            ]))
    } else {
        resp.add_event(Event::new("airdrop").add_attributes(vec![attr("mint_amount", "0"), attr("n_recipients", "0")]))
    })
}
