use crate::{
    error::ContractError,
    math::{add_u128, sub_u128},
    state::storage::RANKED_BALANCES,
};
use cosmwasm_std::{attr, Addr, Api, DepsMut, Env, MessageInfo, Response, Storage, Uint128};
use cw20_base::state::BALANCES;

pub fn create_denom(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attributes(vec![attr("action", "create_denom")]))
}
