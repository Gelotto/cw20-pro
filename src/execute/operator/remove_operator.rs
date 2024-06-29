use crate::{error::ContractError, state::OPERATOR_ADDR};
use cosmwasm_std::{attr, DepsMut, Response};

pub fn exec_remove_operator(deps: DepsMut) -> Result<Response, ContractError> {
    OPERATOR_ADDR.remove(deps.storage);
    Ok(Response::new().add_attributes(vec![attr("action", "remove_operator")]))
}
