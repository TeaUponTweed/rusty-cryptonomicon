use crate::util::{TradingPair};
use crate::rate::{do_optimize_rate};

use std::collections::HashSet;


#[derive(Debug)]
struct NetOptimData {
    asset: String,
    amount : f32,
}

#[derive(Debug)]
pub struct Trade {
    pub exchange: String,
    pub to: String,
    pub to_amount: f32,
    pub from: String,
    pub from_amount: f32,
}


fn get_best_pair(trading_pairs: &Vec<TradingPair>, from: &String, to: &String) -> Option<TradingPair> {
    if let Some(tp) = trading_pairs.iter().filter(|x| &x.base_asset == from && &x.quote_asset == to).min_by(|a,b| a.rate.partial_cmp(&b.rate).unwrap()) {
        Some(tp.clone())
    } else {
        // eprintln!("Failed to find trading pair {} -> {}", from, to);
        None
    }
}

pub fn do_optimize_net(trading_pairs: &Vec<TradingPair>, starting_asset: &String, starting_asset_quantity: f32, final_asset: &String) -> (f32, Vec<Trade>) {
    let mut trading_pairs : Vec<TradingPair> = trading_pairs.iter().map(|x| x.clone()).collect();

    let mut assets_with_movable_currency = vec![NetOptimData{
        asset: starting_asset.clone(),
        amount: starting_asset_quantity
    }];

    let mut trades = Vec::new();

    let mut net_currency = 0.0;
    while let Some(data) = assets_with_movable_currency.pop() {
        // println!("**********");
        // println!("data={:?}", data);
        // println!("assets_with_movable_currency={:?}", assets_with_movable_currency);

        // We have reached the destination asset
        if &data.asset == final_asset {
            net_currency += data.amount;
            continue;
        }

        // There are no valid trades for this currency
        let assets : HashSet<_> = trading_pairs.iter().map(|x| (x.base_asset.clone())).collect();
        // println!("assets={:?}", assets);
        if !assets.contains(&data.asset) {
            continue;
        }

        // No more valid trades (all capacity used up)
        if trading_pairs.len() == 0 {
            continue;
        }

        let (_, path) = do_optimize_rate(&trading_pairs, &data.asset, &final_asset);

        let tp = get_best_pair(&trading_pairs, &path[0], &path[1]).unwrap();
        // Move as much currency down highest rate path that exchange will allow
        let amount_moved = tp.capacity.min(data.amount);
        let trade = Trade{
            exchange: tp.exchange.clone(),
            to: tp.quote_asset.clone(),
            to_amount: amount_moved/tp.rate,
            from: tp.base_asset.clone(),
            from_amount: amount_moved,

        };
        // println!("{:?}", trade);
        trades.push(trade);

        assets_with_movable_currency.push(NetOptimData{
            asset: tp.quote_asset.clone(),
            amount: amount_moved/tp.rate,
        });
        // Remove this trading pair from consideration (TODO this should probably just reduce the capacity, but these cycles are annoying as is)
        trading_pairs = trading_pairs.iter().filter(|x| {
            !(x.exchange == tp.exchange && ((x.quote_asset == tp.quote_asset && x.base_asset  == tp.base_asset)
                                        ||  (x.base_asset  == tp.quote_asset && x.quote_asset == tp.base_asset)))
        }).map(|x| x.clone()).collect();
        
        if tp.capacity < data.amount {

            assets_with_movable_currency.push(NetOptimData{
                asset: data.asset.clone(),
                amount: data.amount - amount_moved,
            });
        }

        // println!("{:?}", assets_with_movable_currency);
        // println!("{:?}", trading_pairs);
        // println!("=======");

    }
    (net_currency, trades)
}

#[cfg(test)]
mod tests {
    use crate::net::*;
    use crate::util::*;
    use crate::util::tests::make_trading_pair_pair;

    #[test]
    fn test_net_optim_simple() {
        let tp1 = make_trading_pair_pair("A".to_string(), "B".to_string(), 0.5, 1.0);
        let tp2 = make_trading_pair_pair("B".to_string(), "C".to_string(), 0.1, 2.0);

        let tps: Vec<TradingPair> = tp1.into_iter().chain(tp2.into_iter()).collect();
        let (net, _) = do_optimize_net(&tps, &"A".to_string(), 1.0, &"B".to_string());
        assert_eq!(net, 2.0);
        let (net, _) = do_optimize_net(&tps, &"A".to_string(), 1.0, &"C".to_string());
        assert_eq!(net, 20.0);
        let (net, _) = do_optimize_net(&tps, &"A".to_string(), 0.5, &"C".to_string());
        assert_eq!(net, 10.0);
        let (net, _) = do_optimize_net(&tps, &"A".to_string(), 2.0, &"C".to_string());
        assert_eq!(net, 20.0);
    }

    #[test]
    fn test_net_optim() {
        let tp1 = make_trading_pair_pair("A".to_string(), "B".to_string(), 0.5, 1.0);
        let tp2 = make_trading_pair_pair("A".to_string(), "C".to_string(), 0.1, 1.0);
        let tp3 = make_trading_pair_pair("E".to_string(), "B".to_string(), 1.0, 100.0);
        let tp4 = make_trading_pair_pair("E".to_string(), "C".to_string(), 1.0, 100.0);
        let tps: Vec<TradingPair> = tp1.into_iter().chain(tp2.into_iter().chain(tp3.into_iter().chain(tp4.into_iter()))).collect();
        let (net, _) = do_optimize_net(&tps, &"A".to_string(), 1.0, &"E".to_string());
        assert_eq!(net, 10.0);
        let (net, _) = do_optimize_net(&tps, &"A".to_string(), 2.0, &"E".to_string());
        assert_eq!(net, 12.0);
        let (net, _) = do_optimize_net(&tps, &"A".to_string(), 10.0, &"E".to_string());
        assert_eq!(net, 12.0);
    }
}
