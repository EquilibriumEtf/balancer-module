use crate::contract::{BalancerApp, BalancerResult};
use crate::msg::{BalancerQueryMsg, ConfigResponse, WeightedAsset};
use crate::state::{ASSET_WEIGHTS, CONFIG};
use abstract_sdk::core::objects::AssetEntry;
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdError, StdResult};

/// Handle queries sent to this app.
pub fn query_handler(
    deps: Deps,
    env: Env,
    _app: &BalancerApp,
    msg: BalancerQueryMsg,
) -> BalancerResult<Binary> {
    match msg {
        BalancerQueryMsg::Config {} => to_binary(&query_config(deps, env)?).map_err(Into::into),
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
