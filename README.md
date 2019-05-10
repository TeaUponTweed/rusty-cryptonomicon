## Asssumtions
* Currency can be effortlessly moved between exchanges, the most favorable rate of all exchanges is used
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

e.g.

```bash
./target/release/cryptoptim rate data/test_data.json B D
```
```
Converting B->D
Optimal conversion rate: 2.3419385 D from 1 B by taking path:
B -> A -> E -> G -> D
```

More specific information can be found by running
```bash
./target/release/cryptoptim {net,rate} -h
```

## Notes
* The net optimization is not optmimal - it uses a greedy rate maximizing heuristic bounded by capacity. I believe this could be solved efficiently using a linear program
* The code is very "Stringly" typed with regards to the Asset names - I'm sure there is a better data model, but I stuck with this path for better or worse

## TODO
* Implement net optimization
