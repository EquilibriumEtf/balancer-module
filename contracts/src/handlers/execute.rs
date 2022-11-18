use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Uint128};

use crate::contract::{BalancerApp, BalancerResult};
use crate::error::BalancerError;
use crate::msg::BalancerExecuteMsg;
use crate::state::{CONFIG, COUNTS};

/// Handle the ` BalancerExecuteMsg`s sent to this app.
pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: BalancerApp,
    msg: BalancerExecuteMsg,
) -> BalancerResult {
    match msg {
        BalancerExecuteMsg::Rebalance {} => commands::rebalance(deps, info, balancer),
        BalancerExecuteMsg::UpdateAssetWeights { to_add, to_remove } => {
            commands::update_asset_weights(deps, info, balancer, to_add, to_remove)
        }
        BalancerExecuteMsg::UpdateConfig { deviation, dex } => {
            commands::update_config(deps, info, balancer, deviation, dex)
        } // ExecuteMsg::SBalanceree { fee } => commands::set_fee(deps, info, dapp, fee),
    }
}
