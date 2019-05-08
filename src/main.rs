// extern crate clap;
// extern crate serde;
// extern crate serde_json;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

// use clap::App; 
use clap::{App, Arg, SubCommand};

use serde::Deserialize;
use serde_json::Result;



#[derive(Debug, Deserialize)]
struct TradingPair {
    exchange: String,
    baseAsset: String,
    quoteAsset: String,
    exchangeRate: f32,
    capacity: f32,
}


fn main() {
    // // Open the path in read-only mode, returns `io::Result<File>`
    // let mut file = match File::open(&path) {
    //     // The `description` method of `io::Error` returns a string that
    //     // describes the error
    //     Err(why) => panic!("couldn't open {}: {}", display,
    //                                                why.description()),
    //     Ok(file) => file,
    // };

    // // Read the file contents into a string, returns `io::Result<usize>`
    // let mut s = String::new();
    // match file.read_to_string(&mut s) {
    //     Err(why) => panic!("couldn't read {}: {}", display,
    //                                                why.description()),
    //     Ok(_) => print!("{} contains:\n{}", display, s),
    // }
    // App::new("SALT Coding Challenge")
    //     .arg(Arg::with_name("trading pairs")
    //                 .help("JSON file containing a list of trading pairs")
    //                 .index(1)
    //                 .required(true)
    //     )
    //     .arg(Arg::with_name("from")
    //                 .help("Starting asset")
    //                 .index(2)
    //                 .required(true)
    //     .arg(Arg::with_name("to")
    //                 .help("Final asset")
    //                 .index(3)
    //                 .required(true)
    //     )
    //     .arg(Arg::with_name("optimize rate")
    //                 .help("Optimize the rate of conversion, ignores capacity")
    //                 .long("optimize-rate")
    //                 .conflicts_with("optimize net")
    //     )
    //     .arg(Arg::with_name("optimize net")
    //                 .help("Optimize the net amount of the final asset.")
    //                 .long("optimize-net")
    //                 .conflicts_with("optimize rate")
    //                 .requires("asset quantity")
    //     )
    //    .about("Given arbitrary crypto-assets --from and --to,"
    //           " this finds a trading-pair chain with the most favorable aggregate exchange rate")
    //    .author("Michael Mason")
    //    .get_matches(); 
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
            )
        )
       .get_matches();

    match matches.subcommand_name() {
        Some("net")  => println!("'cryptoptim add' was used"),
        Some("rate") => println!("'cryptoptim add' was used"),
        _            => println!("Use either \"net\" or \"rate\" subcommand. See help (-h) for details."),
    }
}
