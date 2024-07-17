use crate::checks::ensure_operator;
use crate::error::ContractError;
use crate::execute::before_burn::before_burn;
use crate::execute::before_mint::before_mint;
use crate::execute::before_transfer::before_transfer;
use crate::execute::before_update_marketing::before_update_marketing;
use crate::execute::before_upload_logo::before_upload_logo;
use crate::execute::operator::copy_cw20_balances::exec_copy_cw20_balances;
use crate::execute::operator::freeze::{exec_freeze, exec_unfreeze};
use crate::execute::operator::remove_operator::exec_remove_operator;
use crate::execute::operator::set_operator::exec_set_operator;
use crate::execute::tf::burn::exec_tf_burn;
use crate::execute::tf::derive_balances::exec_tf_derive_balances;
use crate::execute::tf::derive_denom::exec_tf_derive_denom;
use crate::execute::tf::mint::exec_tf_mint;
use crate::execute::tf::remove_denom_admin::exec_tf_remove_admin;
use crate::execute::tf::set_denom_admin::exec_tf_set_admin;
use crate::execute::tf::set_denom_metadata::exec_tf_set_metadata;
use crate::msg::{
    ExecuteMsg, MigrateMsg, OperatorExecuteMsg, ProBalanceQueryMsg, ProQueryMsg, QueryMsg, TokenFactoryExecuteMsg,
};
use crate::query::balances::{query_balances_by_address, query_paginate_balances};
use crate::state;
use cosmwasm_std::{entry_point, to_json_binary};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use cw20_base::allowances::{
    execute_burn_from, execute_decrease_allowance, execute_increase_allowance, execute_send_from,
    execute_transfer_from, query_allowance,
};
use cw20_base::contract::{
    execute_burn, execute_mint, execute_send, execute_transfer, execute_update_marketing, execute_update_minter,
    execute_upload_logo, query_balance, query_download_logo, query_marketing_info, query_minter, query_token_info,
};
use cw20_base::enumerable::{query_all_accounts, query_owner_allowances, query_spender_allowances};

const CONTRACT_NAME: &str = "crates.io:cw20-pro";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const INITIAL_TF_MINT_REPLY_ID: u64 = 1_000_000u64;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: cw20_base::msg::InstantiateMsg,
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
        // Admin msgs
        ExecuteMsg::Pro(msg) => {
            ensure_operator(deps.storage, &info.sender)?;
            match msg {
                OperatorExecuteMsg::RemoveOperator {} => exec_remove_operator(deps),
                OperatorExecuteMsg::SetOperator { address } => exec_set_operator(deps, address),
                OperatorExecuteMsg::FreezeBalances { addresses } => exec_freeze(deps, addresses),
                OperatorExecuteMsg::UnfreezeBalances { addresses } => exec_unfreeze(deps, addresses),
                OperatorExecuteMsg::CopyBalances { cw20_address, mode } => {
                    exec_copy_cw20_balances(deps, cw20_address, mode)
                },
                // TODO: add burner whitelist. if not exist, burning is public
            }
        },

        // TokenFactory msgs
        ExecuteMsg::TokenFactory(msg) => {
            ensure_operator(deps.storage, &info.sender)?;
            match msg {
                TokenFactoryExecuteMsg::Mint { recipients } => exec_tf_mint(deps, env, recipients),
                TokenFactoryExecuteMsg::Burn { amount } => exec_tf_burn(deps, env, amount),
                TokenFactoryExecuteMsg::SetMetadata { metadata } => exec_tf_set_metadata(deps, env, metadata),
                TokenFactoryExecuteMsg::SetAdmin { address } => exec_tf_set_admin(deps, env, address),
                TokenFactoryExecuteMsg::RemoveAdmin {} => exec_tf_remove_admin(deps, env),
                TokenFactoryExecuteMsg::DeriveDenom {} => exec_tf_derive_denom(deps, env),
                TokenFactoryExecuteMsg::DeriveBalances { limit } => exec_tf_derive_balances(deps, env, limit),
            }
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
        ExecuteMsg::Mint { amount, recipient } => {
            before_mint(deps.storage, deps.api, &recipient, amount)?;
            Ok(execute_mint(deps, env, info, recipient, amount)?)
        },
        ExecuteMsg::UpdateMinter { new_minter } => Ok(execute_update_minter(deps, env, info, new_minter)?),
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
        ExecuteMsg::UpateMarketing {
            project,
            description,
            marketing,
        } => {
            let maybe_tf_msg = before_update_marketing(deps.storage, &env, project.to_owned(), description.to_owned())?;
            let resp = execute_update_marketing(deps, env, info, project, description, marketing)?;
            Ok(if let Some(msg) = maybe_tf_msg {
                resp.add_message(msg)
            } else {
                resp
            })
        },
        ExecuteMsg::UploadLogo(logo) => {
            let maybe_tf_msg = before_upload_logo(deps.storage, &env, &logo)?;
            let resp = execute_upload_logo(deps, env, info, logo)?;
            Ok(if let Some(msg) = maybe_tf_msg {
                resp.add_message(msg)
            } else {
                resp
            })
        },
    }
}

// #[entry_point]
// pub fn reply(
//     deps: DepsMut,
//     _env: Env,
//     reply: Reply,
// ) -> Result<Response, ContractError> {
//     Ok(Response::new())
// }

#[entry_point]
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    let result = match msg {
        // Pro tools queries
        QueryMsg::Pro(msg) => match msg {
            // Balance-related queries
            ProQueryMsg::Balances(msg) => match msg {
                ProBalanceQueryMsg::All { limit, desc, cursor } => {
                    to_json_binary(&query_paginate_balances(deps, limit, desc, cursor)?)
                },
                ProBalanceQueryMsg::ByAddress { addresses } => {
                    to_json_binary(&query_balances_by_address(deps, addresses)?)
                },
            },
        },

        // inherited from cw20-base
        QueryMsg::TokenInfo {} => to_json_binary(&query_token_info(deps)?),
        QueryMsg::Balance { address } => to_json_binary(&query_balance(deps, address)?),
        QueryMsg::Minter {} => to_json_binary(&query_minter(deps)?),
        QueryMsg::MarketingInfo {} => to_json_binary(&query_marketing_info(deps)?),
        QueryMsg::DownloadLogo {} => to_json_binary(&query_download_logo(deps)?),
        QueryMsg::Allowance { owner, spender } => to_json_binary(&query_allowance(deps, owner, spender)?),
        QueryMsg::AllAccounts { limit, start_after } => to_json_binary(&query_all_accounts(deps, start_after, limit)?),
        QueryMsg::AllAllowances {
            owner,
            limit,
            start_after,
        } => to_json_binary(&query_owner_allowances(deps, owner, start_after, limit)?),
        QueryMsg::AllSpenderAllowances {
            owner,
            limit,
            start_after,
        } => to_json_binary(&query_spender_allowances(deps, owner, start_after, limit)?),
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
