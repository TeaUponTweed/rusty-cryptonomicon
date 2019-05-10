## Asssumtions
* The tool will not exploit positive cycles when optimizing rate, i.e. a crypto currency will only be visited once
* There are not implicit bi-directional trading pairs e.g. ETH->BTC and BTC->ETH will both be found in the trading pairs file for a given exchange
* Do not need to account for discretization of crypto-currency / this is encoded in the exchange rate

## Installation
From the project root run
```bash
cargo build --release
```
This will place binaries in the `./target` folder

## Input Data Generation
Sample test data can be created by running (requires python3)
```bash
./scripts/gen_data.py > $TRADING_PAIRS
```

## Optimiztion Tool
To run the optimization tool, assuming it has been built, run
```bash
./target/release/cryptoptim {net,rate} $TRADING_PAIRS ...
```
More specific information can be found by running
```bash
./target/release/cryptoptim {net,rate} -h
```

## TODO
* Check for disconnected components in graph
* Implement rate optimization
* Implement net optimization
* Enforce that asset cannot be traded with itself
* Enforce that all pairs are bi-directions