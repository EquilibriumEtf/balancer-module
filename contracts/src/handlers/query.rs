use abstract_sdk::os::objects::AssetEntry;
use crate::contract::BalancerApp;
use crate::msg::{BalancerQueryMsg, ConfigResponse,  WeightedAsset};
use crate::state::{ASSET_WEIGHTS, CONFIG};
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult, StdError};
use cw_storage_plus::Bound;

const DEFAULT_PAGE_SIZE: u8 = 5;
const MAX_PAGE_SIZE: u8 = 20;

/// Handle queries sent to this app.
pub fn query_handler(
    deps: Deps,
    env: Env,
    _app: &BalancerApp,
    msg: BalancerQueryMsg,
) -> StdResult<Binary> {
    match msg {
        BalancerQueryMsg::Config {} => to_binary(&query_config(deps, env)?),
    }
}


pub fn query_config(deps: Deps, _env: Env) -> StdResult<ConfigResponse> {
    let state = CONFIG.load(deps.storage)?;

    let asset_weights: Vec<(AssetEntry, WeightedAsset)> = ASSET_WEIGHTS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<Result<Vec<(AssetEntry, WeightedAsset)>, StdError>>()?;

    Ok(ConfigResponse {
        max_deviation: state.max_deviation,
        asset_weights,
    })
}
