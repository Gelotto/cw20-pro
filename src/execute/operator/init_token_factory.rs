use cosmwasm_std::{attr, DepsMut, Env, Response, SubMsg, Uint256, Uint64};
use cw20::{Logo, MarketingInfoResponse};
use cw20_base::state::{TokenInfo, LOGO, MARKETING_INFO, TOKEN_INFO};

use crate::{
    contract::INITIAL_TF_MINT_REPLY_ID,
    error::ContractError,
    msg::tf::NewDenomMetadata,
    state::tf::{TF_AMOUNT_BURNED, TF_AMOUNT_MINTED, TF_FACTORY, TF_FULL_DENOM, TF_METADATA, TF_REPLY_ID_COUNTER},
    tf::tokenfactory::TokenFactoryType,
};

pub fn exec_token_factory_init(
    deps: DepsMut,
    env: Env,
) -> Result<Response, ContractError> {
    let contract_addr = env.contract.address;

    let TokenInfo {
        decimals, symbol, name, ..
    } = TOKEN_INFO.load(deps.storage)?;

    let MarketingInfoResponse { description, .. } = MARKETING_INFO.load(deps.storage)?;

    // Build metadata params for new bank denom
    let metadata = NewDenomMetadata {
        symbol: symbol.to_owned(),
        name: if name.trim().is_empty() { symbol } else { name },
        decimals: decimals.into(),
        description,
        uri: LOGO
            .may_load(deps.storage)?
            .and_then(|x| if let Logo::Url(url) = x { Some(url) } else { None }),
    };

    // Build the full tokenfactory denom path for new denom
    let subdenom = metadata.symbol.to_lowercase();
    let full_denom = format!("factory/{}/{}", contract_addr, subdenom);

    // Prepare submsgs to create the denom and initalize its metadata
    let factory = TokenFactoryType::from_chain_id(&env.block.chain_id);
    let denom_msgs: Vec<SubMsg> = vec![
        SubMsg::new(factory.create_denom(contract_addr.to_owned(), &subdenom)),
        SubMsg::new(factory.set_denom_metadata(
            contract_addr.to_owned(),
            metadata.to_token_factory_metadata(factory.to_owned(), &full_denom),
        )),
    ];

    // Finish saving finalized state
    TF_FULL_DENOM.save(deps.storage, &full_denom)?;
    TF_FACTORY.save(deps.storage, &factory)?;
    TF_REPLY_ID_COUNTER.save(deps.storage, &Uint64::from(INITIAL_TF_MINT_REPLY_ID))?;
    TF_AMOUNT_MINTED.save(deps.storage, &Uint256::zero())?;
    TF_AMOUNT_BURNED.save(deps.storage, &Uint256::zero())?;
    TF_METADATA.save(deps.storage, &metadata)?;

    Ok(Response::new()
        .add_attributes(vec![attr("action", "tf_init")])
        .add_submessages(denom_msgs))
}
