# BTC value tracker

A Rust application for tracking and visualizing Bitcoin's purchasing power against various real-world commodities using Yahoo Finance data. This allows us to see the value increase of BTC independent from the debasement of fiat currencies since we are tracking purchasing power against real-world goods.

<img width="2880" height="1639" alt="sc" src="https://github.com/user-attachments/assets/9de50e93-9324-4200-b973-38b1fe805b96" />

## Features

- Compare Bitcoin vs real commodities (Gold, Wheat, Corn, Coffee, Beef, Rice)
- Two display modes: 

  - **Units per Currency**: How many units of commodity you can buy with 1 BTC or $1 of USD
  - **Price per Unit**: How much BTC or USD you need to buy 1 unit of commodity
- Date range slider
- data from Yahoo Finance

## Installation

```bash
git clone <repository>
cd asset_price_watcher
cargo build --release
```

The application will automatically fetch data from Yahoo Finance on startup.

## Yahoo Finance Tickers

The app fetches data from these Yahoo Finance tickers:

| Asset | Yahoo Ticker | Notes |
|-------|-------------|--------|
| Bitcoin | BTC-USD | Direct price in USD |
| Gold | GC=F | Gold futures (price per oz) |
| Wheat | ZW=F | Wheat futures (cents/bushel, converted to $/bushel) |
| Corn | ZC=F | Corn futures (cents/bushel, converted to $/bushel) |
| Coffee | KC=F | Coffee futures (cents/lb, converted to $/lb) |
| Beef | LE=F | Live Cattle futures (proxy, cents/lb, converted to $/lb) |
| Rice | ZR=F | Rough Rice futures (cents/cwt, converted to $/cwt) |


### Example Interpretation

If viewing Gold in "Units per Currency" mode:
- Orange line going UP = You can buy MORE consumer goods with 1 BTC
- Blue/Red line going DOWN = You can buy fewer consumer goods with $1
- This demonstrates BTC's increase in purchasing power independent of the debasement of fiat currencies.


## License

MIT
