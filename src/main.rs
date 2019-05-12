use clap::{App, Arg, SubCommand};
use std::path::Path;
use std::process::exit;

mod util;
mod rate;
mod net;

use util::{validate_input, load_trading_pairs};
use rate::do_optimize_rate;
use net::do_optimize_net;

// mod rate;
// mod net;

fn optimize_rate(trading_pair_file: &str, starting_asset: &str, final_asset: &str) {
    let trading_pairs = load_trading_pairs(Path::new(trading_pair_file));

    validate_input(&trading_pairs, starting_asset, final_asset);

    println!("Converting {} -> {}", starting_asset, final_asset);
    let (rate, path) = do_optimize_rate(&trading_pairs, &starting_asset.to_string(), &final_asset.to_string());

    println!("Optimal conversion rate: {} {} from 1 {} by taking path:", rate, final_asset, starting_asset);
    println!("{}", path.join(" -> "));
}


fn optimize_net(trading_pair_file: &str, starting_asset: &str, starting_asset_quantity: f32, final_asset: &str) {
    let trading_pairs = load_trading_pairs(Path::new(trading_pair_file));

    validate_input(&trading_pairs, starting_asset, final_asset);
    if starting_asset_quantity <= 0.0 {
        eprintln!("Starting asset quantity of {} is not valid", starting_asset_quantity);
        exit(1)
    }

    println!("Converting {} {} -> {}", starting_asset_quantity, starting_asset, final_asset);

    let (net, trades) = do_optimize_net(&trading_pairs, &starting_asset.to_string(), starting_asset_quantity, &final_asset.to_string());

    println!("Optimal trading results in {} {} from {} {}.\nThe trades are:", net, final_asset, starting_asset_quantity, starting_asset);
    for trade in trades {
        println!("{} {} -> {} {}", trade.from_amount, trade.from, trade.to_amount, trade.to);
    }
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
