use crate::util::{TradingPair,get_connections,get_rate_map};

use std::collections::HashMap;


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

pub fn do_optimize_rate(trading_pairs: &Vec<TradingPair>, starting_asset: &String, final_asset: &String) -> (f32, Vec<String>) {
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
            if memo_data.cumulative_rate >= data.cumulative_rate
            && is_subset(&memo_data.path, &data.path) {
                continue;
            }
        }
        for next_asset in connections.get(current_asset).unwrap() {
            if !data.path.contains(next_asset){
                let incremental_rate = rate_map.get(&(current_asset.to_string(), next_asset.to_string())).unwrap();
                let cumulative_rate = 1.0/incremental_rate * data.cumulative_rate;
                let mut path : Vec<String> = data.path.iter().map(|x| (x.clone())).collect();
                path.push(next_asset.to_string());
                to_explore.push(RateOptimData{
                    path: path,
                    cumulative_rate: cumulative_rate,
                });
            }
        }

        if !memo.contains_key(current_asset)
        {
            memo.insert(current_asset.to_string(), data);
        }
        else {
            let memo_data = memo.get(current_asset).unwrap();
            if (memo_data.cumulative_rate < data.cumulative_rate)
                || ((memo_data.cumulative_rate == data.cumulative_rate) && (data.path.len() < memo_data.path.len())) {
                memo.insert(current_asset.to_string(), data);
            }
        }
    }
    let final_data = memo.get(final_asset).unwrap();
    (final_data.cumulative_rate, final_data.path.clone())
}

#[cfg(test)]
mod tests {
    use crate::rate::*;
    use crate::util::*;
    use crate::util::tests::make_trading_pair_pair;

    #[test]
    fn test_rate_optim() {
        let tp1 = make_trading_pair_pair("A".to_string(), "B".to_string(), 0.5, 1.0);
        let tp2 = make_trading_pair_pair("B".to_string(), "C".to_string(), 0.1, 1.0);
        let tp3 = make_trading_pair_pair("C".to_string(), "D".to_string(), 0.2, 1.0);
        let mut tps: Vec<TradingPair> = tp1.into_iter().chain(tp2.into_iter().chain(tp3.into_iter())).collect();

        let (rate, path) = do_optimize_rate(&tps, &"A".to_string(), &"B".to_string());
        assert_eq!(rate, 1.0/0.5);
        assert_eq!(path, vec!["A".to_string(), "B".to_string()]);

        let (rate, path) = do_optimize_rate(&tps, &"A".to_string(), &"D".to_string());
        assert_eq!(rate, 1.0/0.5*1.0/0.1*1.0/0.2);
        assert_eq!(path, vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()]);

        let mut tp4 = make_trading_pair_pair("A".to_string(), "E".to_string(), 1.0, 1.0);
        let mut tp5 = make_trading_pair_pair("E".to_string(), "D".to_string(), 0.5*0.1*0.2-0.001, 1.0);
        tps.append(&mut tp4);
        tps.append(&mut tp5);
        let (rate, path) = do_optimize_rate(&tps, &"A".to_string(), &"D".to_string());
        assert_eq!(rate, 111.11111);
        assert_eq!(path, vec!["A".to_string(), "E".to_string(), "D".to_string()]);
    }
}
