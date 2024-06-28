use crate::{
    error::ContractError,
    msg::MintParams,
    state::storage::{FACTORY, FULL_DENOM, MINT_PARAMS, MINT_REPLY_ID_COUNTER},
};
use cosmwasm_std::{attr, Addr, Response, SubMsg, Uint128, Uint64};

use super::Context;

pub fn exec_mint_multiple(
    ctx: Context,
    recipients: Vec<(Addr, Uint128)>,
) -> Result<Response, ContractError> {
    let Context { deps, env, .. } = ctx;
    let denom = FULL_DENOM.load(deps.storage)?;
    let factory = FACTORY.load(deps.storage)?;
    let mut submsgs: Vec<SubMsg> = Vec::with_capacity(recipients.len());

    for (recipient, amount) in recipients.iter() {
        let reply_id = MINT_REPLY_ID_COUNTER
            .update(deps.storage, |n| -> Result<_, ContractError> {
                Ok(n + Uint64::one())
            })?
            .u64()
            - 1;

        MINT_PARAMS.save(
            deps.storage,
            reply_id,
            &MintParams {
                address: recipient.to_owned(),
                amount: amount.to_owned(),
            },
        )?;

        submsgs.push(SubMsg::reply_always(
            factory.mint(
                env.contract.address.to_owned(),
                denom.to_owned(),
                amount.to_owned(),
            ),
            reply_id,
        ))
    }

    Ok(Response::new()
        .add_attributes(vec![attr("action", "mint_multiple")])
        .add_submessages(submsgs))
}
