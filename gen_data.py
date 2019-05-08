import json
import random


_ASSETS=list('ABCDEFG')
_EXCHANGES=list('1234')
_DROP_PROB=0.1

def main():
    values = {asset: 100*random.random() for asset in _ASSETS}
    def gen_trading_pairs():
        for i in range(len(_ASSETS)):
            for j in range(i+1, len(_ASSETS)):
                asset = _ASSETS[i]
                other_asset = _ASSETS[j]
                for exchange in _EXCHANGES:
                    # not all exchanges support all assets
                    if random.random() < _DROP_PROB:
                        continue
                    # noise exchange rate to make things interesting
                    exchange_rate = (values[asset]+random.random())/(values[other_asset] + random.random())
                    # produce a bi-derectional trading pair
                    yield {
                        'exchange': exchange,
                        'quoteAsset': asset,
                        'baseAsset': other_asset,
                        'rate': exchange_rate,
                        'capacity': 50*random.random(),
                    }
                    yield {
                        'exchange': exchange,
                        'quoteAsset': other_asset,
                        'baseAsset': asset,
                        'rate': 1/exchange_rate,
                        'capacity': 50*random.random(),
                    }

    trading_pairs = list(gen_trading_pairs())
    print(json.dumps(trading_pairs))


if __name__ == '__main__':
    main()
