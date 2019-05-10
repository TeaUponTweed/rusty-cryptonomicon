use std::collections::{HashMap,HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use clap::{App, Arg, SubCommand};
use itertools::Itertools;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
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

fn optimize_rate(trading_pair_file: &str, starting_asset: &str, final_asset: &str) {
    let trading_pairs = load_trading_pairs(Path::new(trading_pair_file));
    println!("Converting {}->{}", starting_asset, final_asset);
    for tp in trading_pairs {
        println!("{:?}", tp);
    }
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


fn find_connected_components(trading_pairs: &Vec<TradingPair>) -> Vec<HashSet<String>> {
    let assets = {
        let mut assets = HashMap::new();
        for (base, tps) in &trading_pairs.into_iter().group_by(|tp| tp.base_asset.clone()) {
            let quote_assets : HashSet<String> = tps.into_iter().map(|tp| tp.quote_asset.clone()).collect();
            assets.insert(base, quote_assets);
        }
        assets
    };
    let mut connected_components = Vec::new();
    let mut to_explore : HashSet<String> = assets.keys().map(|x| x.clone()).collect();
    // let mut to_explore : HashSet<String> = assets.keys().collect();

    while let Some(asset) = to_explore.iter().next() {
        let cc = find_connected_component(&assets, asset.to_string());
        to_explore = to_explore.difference(&cc).map(|x| x.clone()).collect();
        println!("{:?}", cc);
        println!("{:?}", to_explore);
        println!("========");
        // for a in cc.iter() {
        //     to_explore.remove(&a);
        // }

        connected_components.push(cc);
    }
    connected_components
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
                println!("'cryptoptim net' was used with quantity = {:?}", quantity);
            }
            else {
                println!("Failed to parse \"{}\" into a float! Exiting now", quantity);
            }
        },
        _ => println!("Use either \"net\" or \"rate\" subcommand. See help (-h) for details."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cc_simple() {
        let tps = vec![
            TradingPair {
                exchange: "1".to_string(),
                quote_asset: "B".to_string(),
                base_asset: "A".to_string(),
                rate: 1.0,
                capacity: 1.0,
            },
            TradingPair {
                exchange: "1".to_string(),
                quote_asset: "A".to_string(),
                base_asset: "B".to_string(),
                rate: 1.0,
                capacity: 1.0,
            },
            TradingPair {
                exchange: "1".to_string(),
                quote_asset: "C".to_string(),
                base_asset: "B".to_string(),
                rate: 1.0,
                capacity: 1.0,
            },
            TradingPair {
                exchange: "1".to_string(),
                quote_asset: "B".to_string(),
                base_asset: "C".to_string(),
                rate: 1.0,
                capacity: 1.0,
            },
            TradingPair {
                exchange: "1".to_string(),
                quote_asset: "D".to_string(),
                base_asset: "C".to_string(),
                rate: 1.0,
                capacity: 1.0,
            },
            TradingPair {
                exchange: "1".to_string(),
                quote_asset: "C".to_string(),
                base_asset: "D".to_string(),
                rate: 1.0,
                capacity: 1.0,
            },
        ];
        let ccs = find_connected_components(&tps);
        println!("{:?}", ccs);
        assert_eq!(ccs.len(), 1);
        assert_eq!(ccs[0].len(), 4);
    }

    #[test]
    fn test_cc_two_components() {
        let tps = vec![
            TradingPair {
                exchange: "1".to_string(),
                quote_asset: "B".to_string(),
                base_asset: "A".to_string(),
                rate: 1.0,
                capacity: 1.0,
            },
            TradingPair {
                exchange: "1".to_string(),
                quote_asset: "A".to_string(),
                base_asset: "B".to_string(),
                rate: 1.0,
                capacity: 1.0,
            },
            TradingPair {
                exchange: "1".to_string(),
                quote_asset: "C".to_string(),
                base_asset: "A".to_string(),
                rate: 1.0,
                capacity: 1.0,
            },
            TradingPair {
                exchange: "1".to_string(),
                quote_asset: "A".to_string(),
                base_asset: "C".to_string(),
                rate: 1.0,
                capacity: 1.0,
            },

            TradingPair {
                exchange: "1".to_string(),
                quote_asset: "D".to_string(),
                base_asset: "E".to_string(),
                rate: 1.0,
                capacity: 1.0,
            },
            TradingPair {
                exchange: "1".to_string(),
                quote_asset: "E".to_string(),
                base_asset: "D".to_string(),
                rate: 1.0,
                capacity: 1.0,
            },
        ];
        let ccs = find_connected_components(&tps);
        println!("{:?}", ccs);
        assert_eq!(ccs.len(), 2);
        assert_eq!(ccs[0].len(), 3);
        assert_eq!(ccs[1].len(), 2);
    }
}