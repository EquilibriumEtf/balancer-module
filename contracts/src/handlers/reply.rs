use cosmwasm_std::{DepsMut, Env, Reply};

use crate::contract::{BalancerModuleApp, BalancerModuleResult};

pub fn example_reply_handler(
    _deps: DepsMut,
    _env: Env,
    _app: BalancerModuleApp,
    _reply: Reply,
) -> BalancerModuleResult {
    // Logic to execute on example reply
    todo!()
}
