use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{BalancerApp, BalancerResult, BALANCER_ID};
use crate::msg::BalancerInstantiateMsg;
use crate::state::{Config, CONFIG};

/// Initial instantiation of the contract
pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: BalancerApp,
    msg: BalancerInstantiateMsg,
) -> BalancerResult {
    // Initial config
    let config: Config = Config {
        dex: msg.dex,
        max_deviation: msg.deviation,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract", BALANCER_ID))
}
