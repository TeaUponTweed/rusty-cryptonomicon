use serde::Deserialize;

use std::process::exit;
use std::fs::File;
use std::io::prelude::*;
use std::collections::{HashMap,HashSet};
use std::path::Path;


#[derive(Debug, Deserialize,Clone)]
#[serde(rename_all = "camelCase")]
pub struct TradingPair {
    pub exchange: String,
    pub quote_asset: String,
    pub base_asset: String,
    pub rate: f32,
    pub capacity: f32,
}


pub fn load_trading_pairs(path: &Path) -> Vec<TradingPair> {
    let mut file = File::open(path).expect("Trading pairs file not found! Exiting");
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let trading_pairs: Vec<TradingPair> =
        serde_json::from_str(&data).expect("JSON was not well-formatted");
    trading_pairs
}

pub fn validate_input(trading_pairs: &Vec<TradingPair>, starting_asset: &str, final_asset: &str) {
    // check that no currencies can be traded for themselves
    if trading_pairs.len() == 0 {
            eprintln!("Trading pairs is empty!");
            exit(1);
    }
    for tp in trading_pairs.iter() {
        if tp.base_asset == tp.quote_asset {
            eprintln!("Asset {} can be traded for itself on exchange {}!", tp.base_asset, tp.exchange);
            exit(1);
        }
    }
    // check that A->B implies B->A
    if !trading_pairs_reversible(&trading_pairs) {
        eprintln!("Not all trading pairs reversive!");
        exit(1);
    }
    // check that starting asset is in graph
    let assets : HashSet<_> = trading_pairs.iter().map(|x| (x.base_asset.clone())).collect();
    if !assets.contains(starting_asset) {
        eprintln!("Asset {} not in trading pairs!", starting_asset);
        exit(1);
    }
    // check that fina; asset is in graph
    if !assets.contains(final_asset) {
        eprintln!("Asset {} not in trading pairs!", final_asset);
        exit(1);
    }
    // check if there exists a path from A->B
    let ccs = find_connected_components(&trading_pairs);
    if ccs.len() > 1 {
        for cc in ccs {
            if cc.contains(starting_asset) && !cc.contains(final_asset) {
                eprintln!("No trading path from {} to {}!", starting_asset, final_asset);
                exit(1);
            }
        }
    }
}

// map each trading edge to its *best* rate
pub fn get_rate_map(trading_pairs: &Vec<TradingPair>) -> HashMap<(String, String), f32> {
    let mut rate_map = HashMap::new();
    for tp in trading_pairs.iter() {
        if let Some(&rate) = rate_map.get(&(tp.base_asset.clone(), tp.quote_asset.clone())) {
            if tp.rate < rate {
                rate_map.insert((tp.base_asset.clone(), tp.quote_asset.clone()), tp.rate);
            }
        }
        else {
            rate_map.insert((tp.base_asset.clone(), tp.quote_asset.clone()), tp.rate);
        }
    }
    rate_map
}


pub fn find_connected_component(connections: &HashMap<String, HashSet<String>>, to_explore_start: String) -> HashSet<String> {
    let mut have_explored = HashSet::new();
    let mut to_explore = Vec::new();
    to_explore.push(to_explore_start);

    while let Some(a) = to_explore.pop() {
        if let Some(next) = connections.get(&a) {
            for unexplored in next.difference(&have_explored) {
                if unexplored != &a {
                    to_explore.push(unexplored.to_string());
                }
            }
        }
        have_explored.insert(a);
    }
    have_explored
}

// Create a mapping from asset -> {next assets}
pub fn get_connections(trading_pairs: &Vec<TradingPair>) -> HashMap<String, HashSet<String>> {
    let mut connections : HashMap<String, HashSet<String>> = HashMap::new();
    for tp in trading_pairs {
        let from = tp.base_asset.clone();
        let to = tp.quote_asset.clone();
        if let Some(reachable_nodes) = connections.get(&from) {
            let mut reachable_nodes : HashSet<String> = reachable_nodes.iter().map(|x| x.clone()).collect();
            reachable_nodes.insert(to);
            connections.insert(from, reachable_nodes);
        }
        else {
            let mut reachable_nodes = HashSet::new();
            reachable_nodes.insert(to);
            connections.insert(from, reachable_nodes);
        }
    }
    connections
}

pub fn find_connected_components(trading_pairs: &Vec<TradingPair>) -> Vec<HashSet<String>> {
    let connections = get_connections(trading_pairs);

    // Find connected components in graph using BFS
    let mut connected_components = Vec::new();
    let mut to_explore : HashSet<String> = connections.keys().map(|x| x.clone()).collect();

    while let Some(asset) = to_explore.iter().next() {
        let cc = find_connected_component(&connections, asset.to_string());
        to_explore = to_explore.difference(&cc).map(|x| x.clone()).collect();
        connected_components.push(cc);
    }
    connected_components
}

fn trading_pairs_reversible(trading_pairs: &Vec<TradingPair>) -> bool {
    let pairs : HashSet<_> = trading_pairs.iter().map(|x| (x.quote_asset.clone(), x.base_asset.clone())).collect();
    for (q, b) in pairs.iter() {
        if !pairs.contains(&(b.to_string(), q.to_string())) {
            return false;
        }
    }
    true
}



#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn make_trading_pair_pair(n1: String, n2: String, rate: f32, capacity: f32) -> Vec<TradingPair> {
        vec![
            TradingPair {
                exchange: "1".to_string(),
                quote_asset: n2.to_string(),
                base_asset: n1.to_string(),
                rate: rate,
                capacity: capacity,
            },
            TradingPair {
                exchange: "1".to_string(),
                quote_asset: n1.to_string(),
                base_asset: n2.to_string(),
                rate: 1.0/rate,
                capacity: capacity,
            },
        ]
    }

    #[test]
    fn test_cc_simple() {
        let tp1 = make_trading_pair_pair("A".to_string(), "B".to_string(), 1.0, 1.0);
        let tp2 = make_trading_pair_pair("B".to_string(), "C".to_string(), 1.0, 1.0);
        let tp3 = make_trading_pair_pair("C".to_string(), "D".to_string(), 1.0, 1.0);
        let tps: Vec<TradingPair> = tp1.into_iter().chain(tp2.into_iter().chain(tp3.into_iter())).collect();
        let ccs = find_connected_components(&tps);
        println!("{:?}", ccs);
        assert_eq!(ccs.len(), 1);
        assert_eq!(ccs[0].len(), 4);
    }

    #[test]
    fn test_cc_two_components() {
        let tp1 = make_trading_pair_pair("A".to_string(), "B".to_string(), 1.0, 1.0);
        let tp2 = make_trading_pair_pair("B".to_string(), "C".to_string(), 1.0, 1.0);
        let tp3 = make_trading_pair_pair("D".to_string(), "E".to_string(), 1.0, 1.0);
        let tps: Vec<TradingPair> = tp1.into_iter().chain(tp2.into_iter().chain(tp3.into_iter())).collect();
        let ccs = {
            let mut ccs = find_connected_components(&tps);
            ccs.sort_by_key(|x| x.len());
            ccs
        };
        println!("{:?}", ccs);
        assert_eq!(ccs.len(), 2);
        assert_eq!(ccs[0].len(), 2);
        assert_eq!(ccs[1].len(), 3);
    }

    #[test]
    fn test_reversive_components() {
        let tp1 = make_trading_pair_pair("A".to_string(), "B".to_string(), 1.0, 1.0);
        let tp2 = make_trading_pair_pair("B".to_string(), "C".to_string(), 1.0, 1.0);
        let mut tps: Vec<TradingPair> = tp1.into_iter().chain(tp2.into_iter()).collect();

        assert!(trading_pairs_reversible(&tps));

        tps.push(TradingPair {
            exchange: "1".to_string(),
            quote_asset: "D".to_string(),
            base_asset: "E".to_string(),
            rate: 1.0,
            capacity: 1.0,
        });
        assert!(!trading_pairs_reversible(&tps));
    }
}
