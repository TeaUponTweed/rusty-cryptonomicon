use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use clap::{App, Arg, SubCommand};

use serde::Deserialize;
// use serde_json::Result;



// #[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize)]
struct TradingPair {
    exchange: String,
    quoteAsset: String,
    baseAsset: String,
    rate: f32,
    capacity: f32,
}



fn load_trading_pairs(path: &Path) -> TradingPair {
    let mut file = File::open(path).expect("Trading pairs file not found! Exiting");
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let trading_pairs: TradingPair =
        serde_json::from_str(&data).expect("JSON was not well-formatted");
    trading_pairs
}

fn optimize_rate(trading_pair_file: &str, starting_asset: &str, final_asset: &str) {
    let trading_pairs = load_trading_pairs(Path::new(trading_pair_file));
    println!("Converting {}->{}", starting_asset, final_asset);
    println!("{:?}", trading_pairs);
    // for tp in trading_pairs {
    //     println!("{:?}", tp);
    // }
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
