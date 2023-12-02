use abstract_sdk::features::AbstractNameService;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{BalancerApp, BalancerResult, BALANCER_ID};
use crate::handlers::{execute, execute_handler};
use crate::msg::BalancerInstantiateMsg;
use crate::state::{Config, CONFIG, TOTAL};

/// Initial instantiation of the contract
pub fn instantiate_handler(
    mut deps: DepsMut,
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
    TOTAL.save(deps.storage, &0u64)?;

    execute::update_asset_weights(deps.branch(), _info, _app, Some(msg.asset_weights), None)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract", BALANCER_ID))
}
