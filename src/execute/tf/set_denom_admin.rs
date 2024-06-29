use crate::{
    error::ContractError,
    state::tf::{TF_FACTORY, TF_FULL_DENOM},
};
use cosmwasm_std::{attr, Addr, DepsMut, Env, Response};

pub fn exec_tf_set_admin(
    deps: DepsMut,
    env: Env,
    new_admin: Addr,
) -> Result<Response, ContractError> {
    let factory = TF_FACTORY.load(deps.storage)?;
    let denom = TF_FULL_DENOM.load(deps.storage)?;

    Ok(Response::new()
        .add_attributes(vec![attr("action", "set_denom_admin")])
        .add_message(factory.change_admin(
            env.contract.address.to_owned(),
            &denom,
            deps.api.addr_validate(new_admin.as_str())?,
        )))
}
