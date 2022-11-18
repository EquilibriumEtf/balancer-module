use cosmwasm_std::{DepsMut, Env, Response};

use crate::contract::{BalancerModuleApp, BalancerModuleResult};

use crate::msg::BalancerModuleMigrateMsg;

/// Unused for now but provided here as an example
/// Contract version is migrated automatically
pub fn migrate_handler(
    _deps: DepsMut,
    _env: Env,
    _app: BalancerModuleApp,
    _msg: BalancerModuleMigrateMsg,
) -> BalancerModuleResult {
    Ok(Response::default())
}
