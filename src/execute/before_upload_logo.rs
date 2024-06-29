use crate::{
    error::ContractError,
    state::tf::{TF_FACTORY, TF_FULL_DENOM, TF_METADATA},
};
use cosmwasm_std::{CosmosMsg, Env, Storage};
use cw20::Logo;

/// Custom business logic that executes before the cw20-base's upload_logo.
pub fn before_upload_logo(
    store: &mut dyn Storage,
    env: &Env,
    logo: &Logo,
) -> Result<Option<CosmosMsg>, ContractError> {
    if let Some(mut metadata) = TF_METADATA.may_load(store)? {
        if let Logo::Url(url) = logo {
            let factory = TF_FACTORY.load(store)?;
            let full_denom = TF_FULL_DENOM.load(store)?;
            let denom_metadata = metadata.to_token_factory_metadata(factory.to_owned(), &full_denom);
            metadata.uri = Some(url.to_owned());
            TF_METADATA.save(store, &metadata)?;
            return Ok(Some(
                factory.set_denom_metadata(env.contract.address.to_owned(), denom_metadata),
            ));
        }
    }
    Ok(None)
}
