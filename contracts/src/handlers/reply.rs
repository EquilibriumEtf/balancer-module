use cosmwasm_std::{DepsMut, Env, Reply};

use crate::contract::{BalancerApp, BalancerResult};

pub fn example_reply_handler(
    _deps: DepsMut,
    _env: Env,
    _app: BalancerApp,
    _reply: Reply,
) -> BalancerResult {
    // Logic to execute on example reply
    todo!()
}
