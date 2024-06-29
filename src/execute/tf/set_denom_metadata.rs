use crate::{
    error::ContractError,
    msg::tf::NewDenomMetadata,
    state::tf::{TF_FACTORY, TF_FULL_DENOM, TF_METADATA},
};
use cosmwasm_std::{attr, DepsMut, Env, Response};

pub fn exec_tf_set_metadata(
    deps: DepsMut,
    env: Env,
    metadata: NewDenomMetadata,
) -> Result<Response, ContractError> {
    let factory = TF_FACTORY.load(deps.storage)?;
    let full_denom = TF_FULL_DENOM.load(deps.storage)?;
    let denom_metadata = metadata.to_token_factory_metadata(factory.to_owned(), &full_denom);

    TF_METADATA.save(deps.storage, &metadata)?;

    Ok(Response::new()
        .add_attributes(vec![attr("action", "set_denom_metadata")])
        .add_message(factory.set_denom_metadata(env.contract.address.to_owned(), denom_metadata)))
}
