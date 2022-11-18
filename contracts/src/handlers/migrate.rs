use cosmwasm_std::{DepsMut, Env, Response};

use crate::contract::{BalancerApp, BalancerResult};

use crate::msg::BalancerMigrateMsg;

/// Unused for now but provided here as an example
/// Contract version is migrated automatically
pub fn migrate_handler(
    _deps: DepsMut,
    _env: Env,
    _app: BalancerApp,
    _msg: BalancerMigrateMsg,
) -> BalancerResult {
    Ok(Response::default())
}
