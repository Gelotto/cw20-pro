use crate::{
    error::ContractError,
    state::{FROZEN_ACCOUNTS, OPERATOR_ADDR},
};
use cosmwasm_std::{ensure_eq, ensure_ne, Addr, Storage};

use crate::state::GLOBAL_BALANCE_FREEZE;

/// Forbid self-transfers, which is something that can be abused to implement
/// certain types of attacks.
pub fn ensure_not_self_transfer(
    sender: &Addr,
    recipient: &Addr,
) -> Result<(), ContractError> {
    ensure_ne!(
        *sender,
        recipient,
        ContractError::Unauthorized {
            reason: "self-transfers not allowed".to_owned()
        }
    );
    Ok(())
}

/// Ensure that there's not global balance freeze AND no freezes on the sender
/// and receiver accounts specifically.
pub fn ensure_accounts_not_frozen(
    store: &dyn Storage,
    sender: Option<Addr>,
    recipient: Option<Addr>,
) -> Result<(), ContractError> {
    // Abort transfer if balances are frozen
    if GLOBAL_BALANCE_FREEZE.may_load(store)?.unwrap_or(false) {
        return Err(ContractError::Unauthorized {
            reason: "CW20 balances are frozen".to_owned(),
        });
    }

    // Abort transfer if sender balance is frozen
    if let Some(sender) = sender {
        if FROZEN_ACCOUNTS.may_load(store, &sender)?.unwrap_or(false) {
            return Err(ContractError::Unauthorized {
                reason: "CW20 sender balance is frozen".to_owned(),
            });
        }
    }

    // Abort transfer if recipient balance is frozen
    if let Some(recipient) = recipient {
        if FROZEN_ACCOUNTS.may_load(store, &recipient)?.unwrap_or(false) {
            return Err(ContractError::Unauthorized {
                reason: "CW20 recipient balance is frozen".to_owned(),
            });
        }
    }

    Ok(())
}

pub fn ensure_operator(
    store: &dyn Storage,
    addr: &Addr,
) -> Result<(), ContractError> {
    ensure_eq!(
        OPERATOR_ADDR.load(store)?,
        *addr,
        ContractError::Unauthorized {
            reason: "Operator authorization required".to_owned()
        }
    );
    Ok(())
}
