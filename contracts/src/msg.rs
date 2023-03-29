//! # App Template
//!
//! `your_namespace::template` is an app which allows users to ...
//!
//! ## Creation
//! The contract can be added on an OS by calling [`ExecuteMsg::CreateModule`](crate::manager::ExecuteMsg::CreateModule) on the manager of the os.
//! ```ignore
//! let template_init_msg = InstantiateMsg:: BalancerInstantiateMsg{
//!               /// The initial value for max_count
//!               pub max_count: Uint128,
//!               /// Initial user counts
//!               pub initial_counts: Option<Vec<(String, Uint128)>>,
//!           };
//!
//! let create_module_msg = ExecuteMsg::CreateModule {
//!                 module: Module {
//!                     info: ModuleInfo {
//!                         name: TEMPLATE.into(),
//!                         version: None,
//!                     },
//!                     kind: crate::core::modules::ModuleKind::External,
//!                 },
//!                 init_msg: Some(to_binary(&template_init_msg).unwrap()),
//!        };
//! // Call create_module_msg on manager
//! ```
//!
//! ## Migration
//! Migrating this contract is done by calling `ExecuteMsg::Upgrade` on [`crate::manager`] with `crate::TEMPLATE` as module.

use abstract_sdk::core::app;
use abstract_sdk::core::objects::AssetEntry;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Decimal;

pub const BALANCER: &str = "equilibrium:balancer";

#[cosmwasm_schema::cw_serde]
pub struct WeightedAsset {
    /// Weight of the asset
    pub weight: u64,
}

/// Migrate msg
#[cosmwasm_schema::cw_serde]
pub struct BalancerMigrateMsg {}

/// Impls for being able to call methods on the autocompounder app directly
impl app::AppExecuteMsg for BalancerExecuteMsg {}
impl app::AppQueryMsg for BalancerQueryMsg {}

/// Init msg
#[cosmwasm_schema::cw_serde]
pub struct BalancerInstantiateMsg {
    /// Weights of the assets in the etf
    pub asset_weights: Vec<(AssetEntry, WeightedAsset)>,
    /// The allowed deviation from the target ratio
    pub deviation: Decimal,
    /// The dex to use for swaps
    pub dex: String,
}

#[cosmwasm_schema::cw_serde]
pub enum BalancerExecuteMsg {
    /// Rebalance the etf
    Rebalance {},
    /// Update asset weights
    UpdateAssetWeights {
        to_add: Option<Vec<(AssetEntry, WeightedAsset)>>,
        to_remove: Option<Vec<AssetEntry>>,
    },
    /// Update config
    UpdateConfig {
        deviation: Option<Decimal>,
        dex: Option<String>,
    },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum BalancerQueryMsg {
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub asset_weights: Vec<(AssetEntry, WeightedAsset)>,
    pub max_deviation: Decimal,
}
