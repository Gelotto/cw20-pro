pub mod models;
pub mod storage;

use cosmwasm_std::{ensure_eq, Addr, DepsMut, Env, MessageInfo, Response};

use crate::{error::ContractError, msg::InstantiateMsg};

/// Top-level initialization of contract state
pub fn init(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("action", "instantiate"))
}

fn ensure_address(
    addr: &Addr,
    exp_addr: &Addr,
    action: &str,
) -> Result<(), ContractError> {
    ensure_eq!(
        *addr,
        *exp_addr,
        ContractError::Unauthorized {
            reason: format!("only nois proxy may execute {}", action)
        }
    );
    Ok(())
}
