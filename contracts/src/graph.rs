use std::collections::{HashMap, HashSet};
use abstract_core::objects::price_source::PriceSource;
use abstract_core::proxy::OracleAsset;
use cw_asset::{AssetInfo, AssetInfoBase};

#[derive(Clone)]
struct AssetInfoWrapper(AssetInfoBase<Addr>);

impl From<AssetInfoBase<Addr>> for AssetInfoWrapper {
    fn from(asset_info: AssetInfoBase<Addr>) -> Self {
        AssetInfoWrapper(asset_info)
    }
}

impl Eq for AssetInfoWrapper;

impl Into<AssetInfoBase<Addr>> for AssetInfoWrapper {
    fn into(self) -> AssetInfoBase<Addr> {
        self.0
    }
}

impl PartialEq for AssetInfoWrapper {
    fn eq(&self, other: &Self) -> bool {
        // Compare based on a unique, identifiable aspect of AssetInfoBase
        self.0.to_string() == other.0.to_string()
    }
}

// impl Eq for AssetInfoWrapper {}

impl Hash for AssetInfoWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash based on the same unique, identifiable aspect
        self.0.to_string().hash(state);
    }
}


struct Graph {
    edges: HashMap<AssetInfoWrapper, HashSet<AssetInfoWrapper>>,
}

impl Graph {
    fn new() -> Self {
        Graph {
            edges: HashMap::new(),
        }
    }

    fn add_pair(&mut self, asset1: AssetInfoBase<Addr>, asset2: AssetInfoBase<Addr>) {
        let asset1_wrapper = AssetInfoWrapper::from(asset1);
        let asset2_wrapper = AssetInfoWrapper::from(asset2);

        self.edges.entry(asset1_wrapper.clone()).or_insert_with(HashSet::new).insert(asset2_wrapper.clone());
        self.edges.entry(asset2_wrapper).or_insert_with(HashSet::new).insert(asset1_wrapper);
    }
    // Additional methods as needed...
}

fn populate_graph(oracle_assets: &[OracleAsset]) -> Graph {
    let mut graph = Graph::new();
    for oracle_asset in oracle_assets {
        match &oracle_asset.price_source {
            PriceSource::Pool { pair, .. } => {
                if pair.len() != 2 {
                    panic!("Pool does not have exactly two assets");
                }
                graph.add_pair(pair[0].clone(), pair[1].clone());
            }
            _ => panic!("Non-pool PriceSource encountered"),
        }
    }
    graph
}

use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use cosmwasm_std::Addr;
use crate::error::BalancerError;

fn find_path(graph: &Graph, start: &AssetInfo, end: &AssetInfo) -> Option<Vec<AssetInfo>> {
    let start = AssetInfoWrapper::from(start.clone());
    let end = AssetInfoWrapper::from(end.clone());
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut prev = HashMap::<AssetInfoWrapper, AssetInfoWrapper>::new();

    visited.insert(start.clone());
    queue.push_back(start.clone());

    while let Some(node) = queue.pop_front() {
        if node == end {
            let mut path = vec![end.clone()];
            let mut current = end;
            while let Some(prev_node) = prev.get(&current) {
                path.push(prev_node.clone());
                current = prev_node.clone();
            }
            path.reverse();
            return Some(path.into_iter().map(Into::into).collect());
        }

        if let Some(neighbors) = graph.edges.get(&node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    visited.insert(neighbor.clone());
                    prev.insert(neighbor.clone(), node.clone());
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    None
}

struct Swap {
    from: AssetInfo,
    to: AssetInfo,
    // Additional fields as needed (like estimated cost, slippage, etc.)
}


// Define the structure for a swap path
struct SwapPath {
    // The sequence of swaps needed to go from the offer asset to the ask asset
    swaps: Vec<Swap>,
}


// Function to determine swap paths
fn determine_swap_paths(
    offer_assets: Vec<AssetInfo>,
    ask_assets: Vec<AssetInfo>,
    oracle_assets: Vec<OracleAsset>, // This is your Vec<OracleAsset>
) -> Result<Vec<SwapPath>, BalancerError> {
    // Step 1: Filter OracleAssets to Pools only and create a graph
    let mut graph = Graph::new();
    for oracle_asset in oracle_assets {
        match oracle_asset.price_source {
            PriceSource::Pool { address, pair } => {
                // Add edges to the graph for each pair in the pool
                // Assuming `pair` has exactly two assets
                graph.add_edge(pair[0].clone(), pair[1].clone(), address);
            }
            _ => panic!("Non-pool PriceSource encountered"),
        }
    }

    // Step 2: Find paths for each offer-ask pair
    let mut swap_paths = Vec::new();
    for offer_asset in offer_assets {
        for ask_asset in ask_assets {
            // Find the best path using the pathfinding algorithm
            let path = find_path(&graph, &offer_asset, &ask_asset)?;
            swap_paths.push(SwapPath {

            });
        }
    }

    // Return the found paths
    Ok(swap_paths)
}

// fn determine_swap_paths(
//     offer_assets: &[AssetInfo],
//     ask_assets: &[AssetInfo],
//     oracle_assets: &[OracleAsset],
// ) -> Vec<Vec<AssetInfo>> {
//     let graph = populate_graph(oracle_assets);
//     let mut swap_paths = Vec::new();
//
//     for offer_asset in offer_assets {
//         for ask_asset in ask_assets {
//             if let Some(path) = find_path(&graph, offer_asset, ask_asset) {
//                 swap_paths.push(path);
//             }
//         }
//     }
//
//     swap_paths
// }
