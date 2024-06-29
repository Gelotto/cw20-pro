use crate::{error::ContractError, state::OPERATOR_ADDR};
use cosmwasm_std::{attr, Addr, DepsMut, Response};

pub fn exec_set_operator(
    deps: DepsMut,
    new_operator: Addr,
) -> Result<Response, ContractError> {
    OPERATOR_ADDR.save(deps.storage, &deps.api.addr_validate(new_operator.as_str())?)?;
    Ok(Response::new().add_attributes(vec![
        attr("action", "set_operator"),
        attr("new_operator", new_operator.to_string()),
    ]))
}
