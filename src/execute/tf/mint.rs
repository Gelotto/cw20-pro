use crate::{
    error::ContractError,
    math::{add_u128, add_u256},
    state::tf::{TF_AMOUNT_MINTED, TF_FACTORY, TF_FULL_DENOM},
};
use cosmwasm_std::{attr, coin, Addr, BankMsg, DepsMut, Env, Response, Storage, SubMsg, Uint128};

pub fn exec_tf_mint(
    deps: DepsMut,
    env: Env,
    recipients: Vec<(Addr, Uint128)>,
) -> Result<Response, ContractError> {
    let resp = Response::new().add_attributes(vec![attr("action", "mint")]);
    Ok(if !recipients.is_empty() {
        let (amount, submsgs) = mint_multiple(deps.storage, &env.contract.address, &recipients)?;
        resp.add_attribute("mint_amount", amount.u128().to_string())
            .add_submessages(submsgs)
    } else {
        resp.add_attribute("mint_amount", "0")
    })
}

// pub fn send_minted_balances(
//     deps: DepsMut,
//     reply: Reply,
// ) -> Result<Response, ContractError> {
//     let mut send_msgs: Vec<SubMsg> = Vec::with_capacity(4);
//     match reply.result {
//         SubMsgResult::Err(e) => return Err(ContractError::Std(StdError::generic_err(e.to_string()))),
//         SubMsgResult::Ok(_) => {
//             let queue_key = format!("_tmp_mint_job_queue_{}", reply.id);
//             let queue: Deque<(Addr, Uint128)> = Deque::new(&queue_key);

//             for _ in 0..queue.len(deps.storage)? {
//                 if let Some((recipient, amount)) = queue.pop_front(deps.storage)? {
//                     let denom = TF_FULL_DENOM.load(deps.storage)?;
//                     send_msgs.push(SubMsg::new(BankMsg::Send {
//                         to_address: recipient.to_string(),
//                         amount: vec![coin(amount.u128(), denom.to_owned())],
//                     }))
//                 }
//             }
//         },
//     }

//     Ok(Response::new().add_submessages(send_msgs))
// }

pub fn mint_multiple(
    store: &mut dyn Storage,
    env_contract_addr: &Addr,
    recipients: &Vec<(Addr, Uint128)>,
) -> Result<(Uint128, Vec<SubMsg>), ContractError> {
    let denom = TF_FULL_DENOM.load(store)?;
    let factory = TF_FACTORY.load(store)?;

    let mut total_mint_amount = Uint128::zero();
    let mut transfer_submsgs: Vec<SubMsg> = Vec::with_capacity(recipients.len());

    for (recipient, amount) in recipients.iter() {
        total_mint_amount = add_u128(total_mint_amount, *amount)?;
        transfer_submsgs.push(SubMsg::new(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![coin(amount.u128(), denom.to_owned())],
        }))
    }

    let mut submsgs = vec![SubMsg::new(factory.mint(
        env_contract_addr.to_owned(),
        denom.to_owned(),
        total_mint_amount.to_owned(),
    ))];

    submsgs.extend(transfer_submsgs);

    TF_AMOUNT_MINTED.update(store, |n| -> Result<_, ContractError> {
        add_u256(n, total_mint_amount)
    })?;

    Ok((total_mint_amount, submsgs))
}
