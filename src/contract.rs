use crate::error::ContractError;
use crate::execute::before_burn::before_burn;
use crate::execute::before_transfer::before_transfer;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, ProQueryMsg, QueryMsg, TokenFactoryExecuteMsg};
use crate::query::query_balances;
use crate::state;
use cosmwasm_std::{entry_point, to_json_binary};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use cw20_base::allowances::{
    execute_burn_from, execute_decrease_allowance, execute_increase_allowance, execute_send_from,
    execute_transfer_from, query_allowance,
};
use cw20_base::contract::{execute_burn, execute_send, execute_transfer, query_balance, query_token_info};

const CONTRACT_NAME: &str = "crates.io:cw20-pro";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(state::init(deps, env, info, msg)?)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // Route tokenfactory-related msgs
        ExecuteMsg::TokenFactory(msg) => match msg {
            TokenFactoryExecuteMsg::Airdrop {} => todo!(),
        },

        // Inherited CW20-base functions
        ExecuteMsg::Transfer { recipient, amount } => {
            before_transfer(deps.storage, deps.api, &info.sender, &recipient, amount)?;
            Ok(execute_transfer(deps, env, info, recipient, amount)?)
        },
        ExecuteMsg::Send { contract, amount, msg } => {
            before_transfer(deps.storage, deps.api, &info.sender, &contract, amount)?;
            Ok(execute_send(deps, env, info, contract, amount, msg)?)
        },
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => {
            before_transfer(
                deps.storage,
                deps.api,
                &deps.api.addr_validate(&owner)?,
                &recipient,
                amount,
            )?;
            Ok(execute_transfer_from(deps, env, info, owner, recipient, amount)?)
        },
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => {
            before_transfer(
                deps.storage,
                deps.api,
                &deps.api.addr_validate(&owner)?,
                &contract,
                amount,
            )?;
            Ok(execute_send_from(deps, env, info, owner, contract, amount, msg)?)
        },
        ExecuteMsg::Burn { amount } => {
            let sender = info.sender.to_string();
            before_burn(deps.storage, deps.api, &sender, amount)?;
            Ok(execute_burn(deps, env, info, amount)?)
        },
        ExecuteMsg::BurnFrom { owner, amount } => {
            before_burn(deps.storage, deps.api, &owner, amount)?;
            Ok(execute_burn_from(deps, env, info, owner, amount)?)
        },
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_increase_allowance(deps, env, info, spender, amount, expires)?),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_decrease_allowance(deps, env, info, spender, amount, expires)?),
    }
}

#[entry_point]
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    let result = match msg {
        QueryMsg::Pro(msg) => match msg {
            ProQueryMsg::Balances { limit, cursor } => to_json_binary(&query_balances(deps, limit, cursor)?),
        },

        // inherited from cw20-base
        QueryMsg::TokenInfo {} => to_json_binary(&query_token_info(deps)?),
        QueryMsg::Balance { address } => to_json_binary(&query_balance(deps, address)?),
        QueryMsg::Allowance { owner, spender } => to_json_binary(&query_allowance(deps, owner, spender)?),
    }?;
    Ok(result)
}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
