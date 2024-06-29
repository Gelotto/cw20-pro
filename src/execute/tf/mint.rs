use crate::{
    error::ContractError,
    math::{add_u128, add_u256},
    state::tf::{TF_AMOUNT_MINTED, TF_FACTORY, TF_FULL_DENOM, TF_REPLY_ID_COUNTER},
};
use cosmwasm_std::{
    attr, Addr, BankMsg, Coin, DepsMut, Env, Event, Reply, Response, StdError, Storage, SubMsg, SubMsgResult, Uint128,
    Uint64,
};
use cw_storage_plus::Deque;

pub fn exec_tf_mint(
    deps: DepsMut,
    env: Env,
    recipients: Vec<(Addr, Uint128)>,
) -> Result<Response, ContractError> {
    let resp = Response::new().add_attributes(vec![attr("action", "mint")]);
    Ok(if !recipients.is_empty() {
        let (amount, transfer_submsg) = mint_multiple(deps.storage, &env.contract.address, &recipients)?;
        resp.add_attribute("mint_amount", amount.u128().to_string())
            .add_submessage(transfer_submsg)
    } else {
        resp.add_attribute("mint_amount", "0")
    })
}

pub fn send_minted_balances(
    deps: DepsMut,
    reply: Reply,
) -> Result<Response, ContractError> {
    let mut send_msgs: Vec<SubMsg> = Vec::with_capacity(4);
    let mut events: Vec<Event> = Vec::with_capacity(4);
    match reply.result {
        SubMsgResult::Err(e) => return Err(ContractError::Std(StdError::generic_err(e.to_string()))),
        SubMsgResult::Ok(_) => {
            let queue_key = format!("_tmp_mint_job_queue_{}", reply.id);
            let queue: Deque<(Addr, Uint128)> = Deque::new(&queue_key);

            for _ in 0..queue.len(deps.storage)? {
                if let Some((recipient, amount)) = queue.pop_front(deps.storage)? {
                    let denom = TF_FULL_DENOM.load(deps.storage)?;

                    events.push(Event::new("mint").add_attributes(vec![
                        attr("amount", amount.u128().to_string()),
                        attr("recipient", recipient.to_string()),
                    ]));

                    send_msgs.push(SubMsg::new(BankMsg::Send {
                        to_address: recipient.to_string(),
                        amount: vec![Coin::new(amount.into(), denom.to_owned())],
                    }))
                }
            }
        },
    }

    Ok(Response::new().add_submessages(send_msgs))
}

pub fn mint_multiple(
    store: &mut dyn Storage,
    env_contract_addr: &Addr,
    recipients: &Vec<(Addr, Uint128)>,
) -> Result<(Uint128, SubMsg), ContractError> {
    let denom = TF_FULL_DENOM.load(store)?;
    let factory = TF_FACTORY.load(store)?;

    let reply_id = TF_REPLY_ID_COUNTER
        .update(store, |n| -> Result<_, ContractError> { Ok(n + Uint64::one()) })?
        .u64()
        - 1;

    // Store each recipient paried with mint amount in storage for use in the reply
    let queue_key = format!("_tmp_mint_job_queue_{}", reply_id);
    let queue: Deque<(Addr, Uint128)> = Deque::new(&queue_key);

    let mut total_mint_amount = Uint128::zero();

    for (recipient, amount) in recipients.iter() {
        queue.push_back(store, &(recipient.to_owned(), *amount))?;
        total_mint_amount = add_u128(total_mint_amount, *amount)?;
    }

    TF_AMOUNT_MINTED.update(store, |n| -> Result<_, ContractError> {
        add_u256(n, total_mint_amount)
    })?;

    Ok((
        total_mint_amount,
        SubMsg::reply_always(
            factory.mint(
                env_contract_addr.to_owned(),
                denom.to_owned(),
                total_mint_amount.to_owned(),
            ),
            reply_id,
        ),
    ))
}
