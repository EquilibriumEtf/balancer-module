use crate::contract::BalancerModuleApp;
use crate::msg::{BalancerModuleQueryMsg, ConfigResponse, UserCountResponse, UserCountsResponse};
use crate::state::{CONFIG, COUNTS};
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Order, StdResult, Uint128};
use cw_storage_plus::Bound;

const DEFAULT_PAGE_SIZE: u8 = 5;
const MAX_PAGE_SIZE: u8 = 20;

/// Handle queries sent to this app.
pub fn query_handler(
    deps: Deps,
    env: Env,
    _app: &BalancerModuleApp,
    msg: BalancerModuleQueryMsg,
) -> StdResult<Binary> {
    match msg {
        BalancerModuleQueryMsg::Config {} => to_binary(&query_config(deps, env)?),
        BalancerModuleQueryMsg::UserCount { user } => to_binary(&query_count(deps, env, user)?),
        BalancerModuleQueryMsg::UserCounts { users } => {
            to_binary(&query_user_counts(deps, env, users)?)
        }
        BalancerModuleQueryMsg::UserCountList {
            page_token,
            page_size,
        } => to_binary(&query_user_count_list(deps, env, page_token, page_size)?),
    }
}

/// Returns the current configuration.
pub fn query_config(deps: Deps, _env: Env) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        max_count: config.max_count,
    })
}

fn user_count(deps: Deps, user: &Addr) -> StdResult<Uint128> {
    let count = COUNTS.may_load(deps.storage, user)?;
    Ok(match count {
        Some(count) => count,
        None => Uint128::zero(),
    })
}

/// Query a single user's count
pub fn query_count(deps: Deps, _env: Env, user: Addr) -> StdResult<UserCountResponse> {
    let count = user_count(deps, &user)?;

    Ok(UserCountResponse { user, count })
}

/// Query a list of users' counts using their addresses
pub fn query_user_counts(deps: Deps, _env: Env, users: Vec<Addr>) -> StdResult<UserCountsResponse> {
    let mut counts: Vec<(Addr, Uint128)> = Vec::new();
    for user in users {
        let count = user_count(deps, &user)?;
        counts.push((user, count));
    }

    Ok(UserCountsResponse { counts })
}

/// Query a list of users' counts
pub fn query_user_count_list(
    deps: Deps,
    _env: Env,
    page_token: Option<String>,
    page_size: Option<u8>,
) -> StdResult<UserCountsResponse> {
    let limit = page_size.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE) as usize;
    let start_bound = page_token.map(|s| Bound::ExclusiveRaw(s.into()));

    let res: Result<Vec<(Addr, Uint128)>, _> = COUNTS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(addr, count)| (addr, count)))
        .collect();

    Ok(UserCountsResponse { counts: res? })
}
