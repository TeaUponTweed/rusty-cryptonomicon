use std::collections::{HashMap,HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;

use clap::{App, Arg, SubCommand};
use serde::Deserialize;


#[derive(Debug, Deserialize,Clone)]
#[serde(rename_all = "camelCase")]
struct TradingPair {
    exchange: String,
    quote_asset: String,
    base_asset: String,
    rate: f32,
    capacity: f32,
}


fn load_trading_pairs(path: &Path) -> Vec<TradingPair> {
    let mut file = File::open(path).expect("Trading pairs file not found! Exiting");
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let trading_pairs: Vec<TradingPair> =
        serde_json::from_str(&data).expect("JSON was not well-formatted");
    trading_pairs
}

fn validate_input(trading_pairs: &Vec<TradingPair>, starting_asset: &str, final_asset: &str) {
    // check that no currencies can be traded for themselves
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

fn optimize_rate(trading_pair_file: &str, starting_asset: &str, final_asset: &str) {
    let trading_pairs = load_trading_pairs(Path::new(trading_pair_file));

    validate_input(&trading_pairs, starting_asset, final_asset);

    println!("Converting {} -> {}", starting_asset, final_asset);
    let (rate, path) = do_optimize_rate(&trading_pairs, &starting_asset.to_string(), &final_asset.to_string());

    println!("Optimal conversion rate: {} {} from 1 {} by taking path:", 1.0/rate, final_asset, starting_asset);
    println!("{}", path.join(" -> "));
}

fn optimize_net(trading_pair_file: &str, starting_asset: &str, starting_asset_quantity: f32, final_asset: &str) {
    let trading_pairs = load_trading_pairs(Path::new(trading_pair_file));

    validate_input(&trading_pairs, starting_asset, final_asset);

    println!("Converting {} {} -> {}", starting_asset_quantity, starting_asset, final_asset);
    let (net, trades) = do_optimize_net(&trading_pairs, &starting_asset.to_string(), starting_asset_quantity, &final_asset.to_string());

    println!("Optimal trading results in {} {} from {} {}. The trades are:", net, final_asset, starting_asset_quantity, starting_asset);
    for trade in trades {
        println!("{} {} -> {} {}", trade.from_amount, trade.from, trade.to_amount, trade.to);
    }
}

#[derive(Debug)]
struct RateOptimData {
    path: Vec<String>,
    cumulative_rate : f32,
}

fn is_subset<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
    for aa in a.iter() {
        if !b.contains(aa) {
            return false;
        }
    }
    return true;
}

// map each trading edge to its *best* rate
fn get_rate_map(trading_pairs: &Vec<TradingPair>) -> HashMap<(String, String), f32> {
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


fn do_optimize_rate(trading_pairs: &Vec<TradingPair>, starting_asset: &String, final_asset: &String) -> (f32, Vec<String>) {
    let connections = get_connections(&trading_pairs);
    let rate_map = get_rate_map(&trading_pairs);

    // initialize problem
    let mut memo : HashMap<String, RateOptimData> = HashMap::new();
    let init_data = RateOptimData{
        path: vec![starting_asset.clone()],
        cumulative_rate: 1.0
    };
    let mut to_explore = vec![init_data];
    // Find optimal rate by exaustive search with memoization
    while let Some(data) = to_explore.pop() {
        let current_asset = data.path.last().unwrap();
        if memo.contains_key(current_asset) {
            let memo_data = memo.get(current_asset).unwrap();
            if memo_data.cumulative_rate <= data.cumulative_rate
            && is_subset(&memo_data.path, &data.path) {
                continue;
            }
        }
        for next_asset in connections.get(current_asset).unwrap() {
            if !data.path.contains(next_asset){
                let incremental_rate = rate_map.get(&(current_asset.to_string(), next_asset.to_string())).unwrap();
                let cumulative_rate = incremental_rate * data.cumulative_rate;
                let mut path : Vec<String> = data.path.iter().map(|x| (x.clone())).collect();
                path.push(next_asset.to_string());
                to_explore.push(RateOptimData{
                    path: path,
                    cumulative_rate: cumulative_rate,
                });
            }
        }
        if !memo.contains_key(current_asset) || (memo.get(current_asset).unwrap().cumulative_rate > data.cumulative_rate) {
            memo.insert(current_asset.to_string(), data);
        }
    }
    let final_data = memo.get(final_asset).unwrap();
    (final_data.cumulative_rate, final_data.path.clone())
}

#[derive(Debug)]
struct NetOptimData {
    asset: String,
    amount : f32,
}

#[derive(Debug)]
struct Trade {
    exchange: String,
    to: String,
    to_amount: f32,
    from: String,
    from_amount: f32,
}

// fn remove_best_pair(trading_pairs: &Vec<TradingPair>. from: String, to: String) -> Option<(TradingPair, Vec<TradingPair>>)> {
//     let best_trading_pair = trading_pairs.iter().filter(|x| x.base_asset == from && x.quote_asset == to).min_by_key(|x| x.rate);
//     match best_trading_pair {
//         Some(best_trading_pair) => Some((best_trading_pair, trading_pairs.iter().filter(|x| != best_trading_pair).collect())),
//         None => None,
//     }
// }

fn get_best_pair(trading_pairs: &Vec<TradingPair>, from: &String, to: &String) -> TradingPair {
    trading_pairs.iter().filter(|x| &x.base_asset == from && &x.quote_asset == to).min_by(|a,b| a.rate.partial_cmp(&b.rate).unwrap()).unwrap().clone()
}

fn do_optimize_net(trading_pairs: &Vec<TradingPair>, starting_asset: &String, starting_asset_quantity: f32, final_asset: &String) -> (f32, Vec<Trade>) {
    let mut trading_pairs : Vec<TradingPair> = trading_pairs.iter().map(|x| x.clone()).collect();

    let mut assets_with_movable_currency = vec![NetOptimData{
        asset: starting_asset.clone(),
        amount: starting_asset_quantity
    }];

    let mut trades = Vec::new();

    let mut net_currency = 0.0;
    while let Some(data) = assets_with_movable_currency.pop() {
        if data.amount <= 0.0 {
            continue
        }

        if &data.asset == final_asset {
            net_currency += data.amount;
        }
        else {
            let (_, path) = do_optimize_rate(&trading_pairs, &data.asset, &final_asset);
            let tp = get_best_pair(&trading_pairs, &path[0], &path[1]);
            let amount_moved = tp.capacity.min(data.amount);

            trades.push(Trade{
                exchange: tp.exchange.clone(),
                to: tp.quote_asset.clone(),
                to_amount: amount_moved/tp.rate,
                from: tp.base_asset.clone(),
                from_amount: amount_moved,

            });

            if tp.capacity < data.amount {
                assets_with_movable_currency.push(NetOptimData{
                    asset: data.asset.clone(),
                    amount: data.amount - amount_moved,
                });
            }
            assets_with_movable_currency.push(NetOptimData{
                asset: tp.quote_asset.clone(),
                amount: amount_moved/tp.rate,
            });
            trading_pairs = trading_pairs.iter().filter(|x| {
                !(x.exchange == tp.exchange && x.quote_asset == tp.quote_asset && x.base_asset == tp.base_asset)
            }).map(|x| x.clone()).collect();
        }
    }
    (net_currency, trades)
}


fn find_connected_component(connections: &HashMap<String, HashSet<String>>, to_explore_start: String) -> HashSet<String> {
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
fn get_connections(trading_pairs: &Vec<TradingPair>) -> HashMap<String, HashSet<String>> {
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

fn find_connected_components(trading_pairs: &Vec<TradingPair>) -> Vec<HashSet<String>> {
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


fn main() {
    let matches = App::new("cryptoptim")
        .subcommand(SubCommand::with_name("net")
            .about("Optimize the net amount of the final asset")
            .arg(Arg::with_name("trading pairs")
                .help("JSON file containing a list of trading pairs")
                .index(1)
                .required(true)
            )
            .arg(Arg::with_name("from")
                .help("Starting asset")
                .index(2)
                .required(true)
            )
            .arg(Arg::with_name("to")
                .help("Destination asset")
                .index(3)
                .required(true)
            )
            .arg(Arg::with_name("asset quantity")
                .help("Amount of starting asset")
                .index(4)
                .required(true)
            )
        )
        .subcommand(SubCommand::with_name("rate")
            .about("Optimize the rate of asset conversion. Ignores capacity")
            .arg(Arg::with_name("trading pairs")
                .help("JSON file containing a list of trading pairs")
                .index(1)
                .required(true)
            )
            .arg(Arg::with_name("from")
                .help("Starting asset")
                .index(2)
                .required(true)
            )
            .arg(Arg::with_name("to")
                .help("Destination asset")
                .index(3)
                .required(true)
            )
        )
       .get_matches();

    match matches.subcommand() {
        ("rate", Some(rate_matches))  => {
            optimize_rate(
                rate_matches.value_of("trading pairs").unwrap(),
                rate_matches.value_of("from").unwrap(),
                rate_matches.value_of("to").unwrap()
            );
        },
        ("net", Some(net_matches)) => {
            let quantity = net_matches.value_of("asset quantity").unwrap();
            if let Some(quantity) = quantity.parse::<f32>().ok() {
                optimize_net(
                    net_matches.value_of("trading pairs").unwrap(),
                    net_matches.value_of("from").unwrap(),
                    quantity,
                    net_matches.value_of("to").unwrap()
                )
            }
            else {
                eprintln!("Failed to parse \"{}\" into a float!", quantity);
            }
        },
        _ => eprintln!("Use either \"net\" or \"rate\" subcommand. See help (-h) for details."),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    fn make_trading_pair_pair(n1: String, n2: String, rate: f32, capacity: f32) -> Vec<TradingPair> {
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

    #[test]
    fn test_rate_optim() {
        let tp1 = make_trading_pair_pair("A".to_string(), "B".to_string(), 0.5, 1.0);
        let tp2 = make_trading_pair_pair("B".to_string(), "C".to_string(), 0.1, 1.0);
        let tp3 = make_trading_pair_pair("C".to_string(), "D".to_string(), 0.2, 1.0);
        let mut tps: Vec<TradingPair> = tp1.into_iter().chain(tp2.into_iter().chain(tp3.into_iter())).collect();

        let (rate, path) = do_optimize_rate(&tps, "A".to_string(), "B".to_string());
        assert_eq!(rate, 0.5);
        assert_eq!(path, vec!["A".to_string(), "B".to_string()]);

        let (rate, path) = do_optimize_rate(&tps, "A".to_string(), "D".to_string());
        assert_eq!(rate, 0.5*0.1*0.2);
        assert_eq!(path, vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()]);

        let mut tp4 = make_trading_pair_pair("A".to_string(), "E".to_string(), 1.0, 1.0);
        let mut tp5 = make_trading_pair_pair("E".to_string(), "D".to_string(), 0.5*0.1*0.2 - 0.0001, 1.0);
        tps.append(&mut tp4);
        tps.append(&mut tp5);
        let (rate, path) = do_optimize_rate(&tps, "A".to_string(), "D".to_string());
        assert_eq!(rate, 0.5*0.1*0.2-0.0001);
        assert_eq!(path, vec!["A".to_string(), "E".to_string(), "D".to_string()]);
    }
}