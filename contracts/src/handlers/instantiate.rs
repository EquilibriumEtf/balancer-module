use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::contract::{BalancerApp, BalancerResult, APP_NAME};

use crate::msg::BalancerInstantiateMsg;
use crate::state::{Config, CONFIG, COUNTS};

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
        max_count: msg.max_count,
    };

    CONFIG.save(deps.storage, &config)?;

    if let Some(initial_counts) = msg.initial_counts {
        for (addr, count) in initial_counts {
            let addr = deps.api.addr_validate(addr.as_str())?;

            COUNTS.save(deps.storage, &addr, &count)?;
        }
    }

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract", APP_NAME))
}
