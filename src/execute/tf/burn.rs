use crate::{
    error::ContractError,
    state::tf::{TF_AMOUNT_BURNED, TF_FACTORY, TF_FULL_DENOM},
};
use cosmwasm_std::{attr, DepsMut, Env, Response, StdError, Uint128};

pub fn exec_tf_burn(
    deps: DepsMut,
    env: Env,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let factory = TF_FACTORY.load(deps.storage)?;
    let denom = TF_FULL_DENOM.load(deps.storage)?;

    TF_AMOUNT_BURNED.update(deps.storage, |n| -> Result<_, ContractError> {
        Ok(n.checked_add(amount.into())
            .map_err(|e| ContractError::Std(StdError::overflow(e)))?)
    })?;

    Ok(Response::new()
        .add_attributes(vec![attr("action", "burn")])
        .add_message(factory.burn(env.contract.address.to_owned(), &denom, amount)))
}
