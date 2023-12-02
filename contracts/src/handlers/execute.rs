use std::collections::BTreeMap;
use std::iter;
use std::str::FromStr;
use abstract_core::objects::AnsAsset;
use abstract_dex_adapter::DexInterface;

use abstract_sdk::core::objects::AssetEntry;
use abstract_sdk::core::proxy::OracleAsset;
use abstract_sdk::features::AbstractNameService;
use abstract_sdk::{AccountingInterface};
use cosmwasm_std::{Addr, Decimal, Decimal256, Deps, DepsMut, ensure, Env, MessageInfo, Response, StdResult, Uint128, Uint256};
use cw_asset::AssetInfo;
use abstract_dex_adapter::msg::{AskAsset, OfferAsset};

use crate::contract::{BalancerApp, BalancerResult};
use crate::error::BalancerError;
use crate::msg::{BalancerExecuteMsg, WeightedAsset};
use crate::state::{ASSET_WEIGHTS, CONFIG, TOTAL};

const MINIMUM_WEIGHT_THRESHOLD: u64 = 1;

/// Handle the ` BalancerExecuteMsg`s sent to this app.
pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    balancer: BalancerApp,
    msg: BalancerExecuteMsg,
) -> BalancerResult {
    match msg {
        BalancerExecuteMsg::Rebalance {} => rebalance(deps, info, balancer),
        BalancerExecuteMsg::UpdateAssetWeights { to_add, to_remove } => {
            update_asset_weights(deps, info, balancer, to_add, to_remove)
        }
        BalancerExecuteMsg::UpdateConfig { deviation, dex } => {
            update_config(deps, info, balancer, deviation, dex)
        } // ExecuteMsg::SetFee { fee } => commands::set_fee(deps, info, dapp, fee),
    }
}

/// Take the assets in the basket, calculate their deviations from the expected weights, and call upon the dex to perform the swaps
pub fn rebalance(deps: DepsMut, _msg_info: MessageInfo, balancer: BalancerApp) -> BalancerResult {
    let config = CONFIG.load(deps.storage)?;
    let dex = config.dex;
    let max_deviation = config.max_deviation;

    let accountant = balancer.accountant(deps.as_ref());

    let base_state = balancer.load_state(deps.storage)?;

    // Retrieve the enabled assets
    let assets_list: (Vec<AssetInfo>, Vec<OracleAsset>) =
        accountant.assets_list()?.assets.into_iter().unzip();
    let assets = assets_list.0.clone();

    println!("Assets: {:?}", assets);

    // if there are no assets, no action is necessary
    if assets.is_empty() {
        return Ok(Response::default());
    }

    // reverse lookup for the asset names
    let ans_host = balancer
        .ans_host(deps.as_ref())?;

    let asset_names = ans_host
        .query_assets_reverse(&deps.querier, &assets)?;

    let assets = BTreeMap::from_iter(iter::zip(asset_names, assets));
    let (offer_assets, ask_assets) = determine_assets_to_swap(
        &deps,
        &balancer,
        base_state.proxy_address,
        max_deviation,
        assets,
    )?;

    deps.api.debug(&format!("Offer: {:?}", offer_assets.clone()));
    deps.api.debug(&format!("Ask: {:?}", offer_assets.clone()));


    let exchange = balancer.dex(deps.as_ref(), dex);

    let base_asset = accountant.base_asset()?;
    let base_asset = ans_host.query_asset_reverse(&deps.querier, &base_asset.base_asset)?;

    // sell all offers for base, and buy all asks with base
    let mut swaps = vec![];

    for offer in offer_assets {
        // let simulate = exchange.simulate_swap(offer, base_asset.clone())?;

        swaps.push(exchange.swap(offer, base_asset.clone(), None, None)?);
    }

    for ask in ask_assets {
        // simulate swapping ask for base, then do reverse swap with amount?
        let simulate = exchange.simulate_swap(ask.clone(), base_asset.clone())?;

        deps.api.debug(&format!("Simulated: {:?}", simulate.clone()));

        swaps.push(exchange.swap(AnsAsset {
            name: base_asset.clone(),
            amount: simulate.return_amount,
        }, ask.name.clone(), Some(Decimal::new(simulate.spread_amount)), None)?)
    }

    Ok(Response::new().add_messages(swaps))
}

/// Return arrays of offer and ask assets (those to sell and those to buy)
fn determine_assets_to_swap(
    deps: &DepsMut,
    balancer: &BalancerApp,
    proxy_address: Addr,
    max_deviation: Decimal,
    assets: BTreeMap<AssetEntry, AssetInfo>,
) -> Result<(Vec<OfferAsset>, Vec<AskAsset>), BalancerError> {
    let accountant = balancer.accountant(deps.as_ref());
    // Get the total value of the assets
    let account_Value = accountant.query_total_value()?.total_value.amount;

    // Empty accounts are fully balanced
    if account_Value.is_zero() {
        return Err(BalancerError::ValuelessAccount {}.into());
    }

    let mut offer_assets: Vec<OfferAsset> = Vec::new();
    let mut ask_assets: Vec<AskAsset> = Vec::new();

    for (asset_entry, asset_info) in assets {
        // actual weight of the asset (percent)
        let actual_weight = calc_actual_weight(deps, &balancer, account_Value, asset_entry.clone())?;

        deps.api.debug(&format!("Asset: {:?}, weight: {:?}", asset_entry.clone(), actual_weight.clone()));

        let asset_balance = Decimal256::from_atomics(
            Uint256::from_uint128(asset_info.query_balance(&deps.querier, &proxy_address)?),
            6,
        )
            .unwrap();

        // Load the expected normalized weight (percentage)
        let expected_weight = calc_normalized_weight(deps.as_ref(), asset_entry.clone())?;

        if actual_weight.is_zero() {
            let single_token_value = accountant.asset_value(asset_entry.clone())?;

            deps.api.debug(&format!("single_token_value: {:?}", single_token_value.clone()));

            // Weight of a single asset (in percent)
            // Ex: 1 JUNO == 5, Pool == 20, 5/20 = 0.25 (25%)
            let single_asset_weight = Decimal256::from_ratio(single_token_value, account_Value);

            deps.api.debug(&format!("single_asset_weight: {:?}", single_asset_weight.clone()));

            // Expected
            // Ex: Expected: 0.50 (50%) / 0.25 (25%) = 2 tokens to buy - existing balance (likely close to zero)
            let buy_amount = expected_weight
                .checked_div(single_asset_weight)?;

            let amount: Uint128 =
                Decimal::from_str(&buy_amount.to_string()).unwrap() * Uint128::one();

            ask_assets.push(AnsAsset::new(asset_entry, amount));
            continue
        }

        // (Expected / Actual) Weight
        let weight_ratio: Decimal256 = expected_weight.checked_div(actual_weight)?;

        // Skip the asset if it doesn't need rebalancing
        if expected_weight
            .abs_diff(actual_weight)
            .lt(&max_deviation.into())
        {
            continue;
        }

        // Absolute percentage change in balance
        let change_in_balance = weight_ratio.abs_diff(Decimal256::one());

        if expected_weight.gt(&actual_weight) {
            // The asset is overweight and must be sold
            // Sell (1 - (expected / actual)) * balance of the asset
            // If the weight_ratio is within reason we don't need to query the explicit value of the asset
            // We ceil to ensure that we can afford to buy the other assets
            let sell_amount = asset_balance.checked_mul(change_in_balance)?.ceil();

            // Hacky way of converting Decimal256 to Uint128
            let offer_amount: Uint128 =
                Decimal::from_str(&sell_amount.to_string()).unwrap() * Uint128::one();
            offer_assets.push(AskAsset::new(asset_entry, offer_amount));
        } else {
            // The asset is underweight and must be bought
            // Buy ((expected / actual) - 1) * balance of the asset

            // If we are below the minimum threshold, our fixed_point calculation could be off due to overflow,
            // so we need to query the explicit value of the asset
            let buy_amount = if actual_weight.lt(&Decimal256::percent(MINIMUM_WEIGHT_THRESHOLD)) {
                let single_token_value = accountant.asset_value(asset_entry.clone())?;

                // Weight of a single asset (in percent)
                // Ex: 1 JUNO == 5, Pool == 20, 5/20 = 0.25 (25%)
                let single_asset_weight = Decimal256::from_ratio(single_token_value, account_Value);
                // Expected
                // Ex: Expected: 0.50 (50%) / 0.25 (25%) = 2 tokens to buy - existing balance (likely close to zero)
                expected_weight
                    .checked_div(single_asset_weight)?
                    // Subtract the existing balance
                    .checked_sub(asset_balance)?
            } else {
                // We floor to ensure we don't buy more than we can afford
                asset_balance.checked_mul(change_in_balance)?.floor()
            };
            let amount: Uint128 =
                Decimal::from_str(&buy_amount.to_string()).unwrap() * Uint128::one();
            ask_assets.push(AskAsset::new(asset_entry, amount));
        };
    }
    Ok((offer_assets, ask_assets))
}

/// Calculate the actual weight of the asset
/// (asset(s) value / total pool value) * 100
fn calc_actual_weight(
    deps: &DepsMut,
    balancer: &BalancerApp,
    pool_value: Uint128,
    asset_name: AssetEntry,
) -> Result<Decimal256, BalancerError> {
    let holding_value = balancer.accountant(deps.as_ref()).asset_value(asset_name)?;

    // Actual weight in percentage
    let actual_weight = Decimal256::from_ratio(holding_value, pool_value);
    Ok(actual_weight)
}

/// Returns the normalized weight (percentage) of the provided asset
fn calc_normalized_weight(deps: Deps, asset: AssetEntry) -> Result<Decimal256, BalancerError> {
    let asset_weight = ASSET_WEIGHTS
        .may_load(deps.storage, &asset)?
        .ok_or(BalancerError::WrongToken {})?
        .weight;

    let total_weight = TOTAL.load(deps.storage)?;

    Ok(Decimal256::from_ratio(asset_weight, total_weight))
}

/// Update the configured asset weights
/// @todo - AssetWeightChangedHook
pub fn update_asset_weights(
    deps: DepsMut,
    _msg_info: MessageInfo,
    dapp: BalancerApp,
    to_add: Option<Vec<(AssetEntry, WeightedAsset)>>,
    to_remove: Option<Vec<AssetEntry>>,
) -> BalancerResult {
    let mut total = TOTAL.load(deps.storage)?;

    // Remove the assets and their weights
    if let Some(assets_to_remove) = to_remove {
        for asset_to_remove in assets_to_remove.into_iter() {
            // Check if the asset is present
            if !ASSET_WEIGHTS.has(deps.storage, &asset_to_remove.clone()) {
                return Err(BalancerError::AssetNotPresent {
                    asset: asset_to_remove.clone().to_string(),
                });
            }
            let prev_weight = ASSET_WEIGHTS
                .load(deps.storage, &asset_to_remove.clone())?
                .weight;
            // Remove the weight
            ASSET_WEIGHTS.remove(deps.storage, &asset_to_remove);
            total -= prev_weight;
        }
    }

    // Add the new weights
    if let Some(new_weights) = to_add {
        // check that all the to_add exist in ans
        let new_entries = new_weights.clone().into_iter().map(|(e, _)| e.clone()).collect::<Vec<AssetEntry>>();
        let resolved_entries = dapp.ans_host(deps.as_ref())?.query_assets(&deps.querier, &new_entries)?;

        let registered_assets: Vec<AssetInfo> = dapp.accountant(deps.as_ref()).assets_list()?.assets.into_iter().map(|(a, _)| a).collect();
        for resolved in resolved_entries {
            ensure!(registered_assets.contains(&resolved), BalancerError::AssetNotRegistered { asset: resolved });
        }

        for new_weight in new_weights.into_iter() {
            ASSET_WEIGHTS.update(
                deps.storage,
                &new_weight.0,
                |_| -> StdResult<WeightedAsset> { Ok(new_weight.1.clone()) },
            )?;
            total += new_weight.1.weight;
        }
    }

    TOTAL.save(deps.storage, &total)?;

    Ok(Response::new().add_attribute("action", "update asset weights"))
}

/// Asserts that the 0.5 < percent <= 1.0
fn validate_deviation(percent: &Decimal) -> Result<(), BalancerError> {
    if *percent > Decimal::percent(100) || *percent < Decimal::percent(1) {
        return Err(BalancerError::InvalidDeviation {
            deviation: percent.clone(),
        });
    }
    Ok(())
}

pub fn update_config(
    deps: DepsMut,
    _msg_info: MessageInfo,
    _dapp: BalancerApp,
    max_deviation: Option<Decimal>,
    dex: Option<String>,
) -> BalancerResult {
    // Load the current config
    let mut config = CONFIG.load(deps.storage)?;

    // Update the config
    if let Some(new_max_deviation) = max_deviation {
        validate_deviation(&new_max_deviation)?;
        config.max_deviation = new_max_deviation;
    }

    if let Some(dex) = dex {
        config.dex = dex;
    }

    // Save the new config
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update config"))
}
