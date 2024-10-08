use crate::{
    error::ContractError,
    state::tf::{TF_FACTORY, TF_FULL_DENOM},
};
use cosmwasm_std::{attr, Addr, CanonicalAddr, DepsMut, Env, Response};

pub fn exec_tf_remove_admin(
    deps: DepsMut,
    env: Env,
) -> Result<Response, ContractError> {
    let factory = TF_FACTORY.load(deps.storage)?;
    let denom = TF_FULL_DENOM.load(deps.storage)?;

    // apparently, one removes admin by setting admin to "null address"
    let empty_canonical_addr = CanonicalAddr::from(vec![]);
    let empty_addr = Addr::unchecked(deps.api.addr_humanize(&empty_canonical_addr)?);

    Ok(Response::new()
        .add_attributes(vec![attr("action", "remove_denom_admin")])
        .add_message(factory.change_admin(env.contract.address.to_owned(), &denom, empty_addr)))
}
