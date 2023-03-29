use abstract_sdk::core::objects::AssetEntry;
use cosmwasm_std::Decimal;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::msg::WeightedAsset;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
/// State stores LP token address
/// BaseState is initialized in contract
pub struct Config {
    // the allowed deviation from the target ratio
    pub max_deviation: Decimal,
    // the dex to use for swaps
    pub dex: String,
}

pub const CONFIG: Item<Config> = Item::new("\u{0}{5}config");
pub const TOTAL: Item<u64> = Item::new("\u{0}{5}total");

// Could use a SnapshotMap
pub const ASSET_WEIGHTS: Map<&AssetEntry, WeightedAsset> = Map::new("\u{0}{6}assets");
