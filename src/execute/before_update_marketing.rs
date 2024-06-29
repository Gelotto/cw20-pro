use crate::{
    error::ContractError,
    state::tf::{TF_FACTORY, TF_FULL_DENOM, TF_METADATA},
};
use cosmwasm_std::{CosmosMsg, Env, Storage};

/// Custom business logic that executes before the cw20-base's update_marketing.
pub fn before_update_marketing(
    store: &mut dyn Storage,
    env: &Env,
    project: Option<String>,
    description: Option<String>,
) -> Result<Option<CosmosMsg>, ContractError> {
    try_update_token_factory_metadata(store, env, project, description)
}

fn try_update_token_factory_metadata(
    store: &mut dyn Storage,
    env: &Env,
    project: Option<String>,
    description: Option<String>,
) -> Result<Option<CosmosMsg>, ContractError> {
    if let Some(mut metadata) = TF_METADATA.may_load(store)? {
        if let Some(project) = project {
            metadata.name = project.trim().to_owned();
        }
        if let Some(desc) = description {
            let desc = desc.trim().to_owned();
            if desc.is_empty() {
                metadata.description = None;
            } else {
                metadata.description = Some(desc)
            }
        }
        let factory = TF_FACTORY.load(store)?;
        let full_denom = TF_FULL_DENOM.load(store)?;
        let denom_metadata = metadata.to_token_factory_metadata(factory.to_owned(), &full_denom);
        return Ok(Some(
            factory.set_denom_metadata(env.contract.address.to_owned(), denom_metadata),
        ));
    }
    Ok(None)
}
