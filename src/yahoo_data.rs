use yahoo_finance_api as yahoo;
use chrono::{DateTime, Utc, TimeZone};
use yahoo_finance_api::time::OffsetDateTime;
use std::collections::HashMap;
use anyhow::Result;
use crate::models::Asset;
use crate::data::PriceData;

pub struct YahooDataFetcher {
    provider: yahoo::YahooConnector,
}

impl YahooDataFetcher {
    pub fn new() -> Self {
        Self {
            provider: yahoo::YahooConnector::new().unwrap(),
        }
    }

    fn get_ticker_for_asset(asset: Asset) -> Option<&'static str> {
        match asset {
            Asset::Bitcoin => Some("BTC-USD"),
            Asset::Gold => Some("GC=F"),      // Gold futures (price per oz)
            Asset::Silver => Some("SI=F"),    // Silver futures (price per oz)
            Asset::Oil => Some("CL=F"),       // Crude Oil WTI futures (price per barrel)
            Asset::NaturalGas => Some("NG=F"), // Natural Gas futures (price per MMBtu)
            Asset::Copper => Some("HG=F"),     // Copper futures (cents per lb, need /100)
            Asset::Wheat => Some("ZW=F"),     // Wheat futures (cents per bushel, need /100)
            Asset::Corn => Some("ZC=F"),      // Corn futures (cents per bushel, need /100)
            Asset::Soybeans => Some("ZS=F"),  // Soybeans futures (cents per bushel, need /100)
            Asset::Coffee => Some("KC=F"),    // Coffee futures (cents per lb, need /100)
            Asset::Sugar => Some("SB=F"),     // Sugar futures (cents per lb, need /100)
            Asset::Cotton => Some("CT=F"),    // Cotton futures (cents per lb, need /100)
            Asset::Beef => Some("LE=F"),      // Live Cattle futures (cents per lb, need /100)
            Asset::Rice => Some("ZR=F"),      // Rough Rice futures (cents per cwt, need /100)
            Asset::Lumber => Some("LBS=F"),   // Lumber futures (price per 1000 bd ft)
            Asset::ConsumerBasket => None,    // Will be calculated from other assets
        }
    }
    
    pub async fn fetch_historical_data(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<HashMap<Asset, Vec<PriceData>>> {
        let mut all_data = HashMap::new();
        
        // Add delay between requests to avoid rate limiting
        use tokio::time::{sleep, Duration};
        
        // Fetch USD/CAD rates first using direct HTTP (yahoo_finance_api has issues with CAD=X)
        let usd_cad_rates = match self.fetch_ticker_direct("CAD=X", start_date, end_date).await {
            Ok(rates) => {
                println!("Successfully fetched {} USD/CAD rates via direct HTTP", rates.len());
                rates
            }
            Err(e) => {
                eprintln!("Failed to fetch USD/CAD rates: {}", e);
                Vec::new()
            }
        };
        
        // Get all assets except ConsumerBasket (CAD=X is fetched separately above)
        let all_tickers = vec![
            ("BTC-USD", Some(Asset::Bitcoin)),
            ("GC=F", Some(Asset::Gold)),
            ("SI=F", Some(Asset::Silver)),
            ("CL=F", Some(Asset::Oil)),
            ("NG=F", Some(Asset::NaturalGas)),
            ("HG=F", Some(Asset::Copper)),
            ("ZW=F", Some(Asset::Wheat)),
            ("ZC=F", Some(Asset::Corn)),
            ("ZS=F", Some(Asset::Soybeans)),
            ("KC=F", Some(Asset::Coffee)),
            ("SB=F", Some(Asset::Sugar)),
            ("CT=F", Some(Asset::Cotton)),
            ("LE=F", Some(Asset::Beef)),
            ("ZR=F", Some(Asset::Rice)),
            ("LBS=F", Some(Asset::Lumber)),
        ];
        
        // Fetch all data in one loop with consistent delays
        for (i, (ticker, asset_opt)) in all_tickers.iter().enumerate() {
            // Add delay between ALL requests (including first one)
            if i > 0 {
                println!("Waiting 3 seconds before next request to avoid rate limiting...");
                sleep(Duration::from_secs(3)).await;
            }
            
            println!("Fetching data for {} ({})", 
                     asset_opt.map(|a| format!("{:?}", a)).unwrap_or("USD/CAD".to_string()), 
                     ticker);
            
            match self.fetch_asset_data(ticker, start_date, end_date).await {
                Ok(mut data) => {
                    println!("Successfully fetched {} data points", data.len());
                    
                    if let Some(asset) = asset_opt {
                        // For Bitcoin, note that data only goes back to Sept 2014
                        if *asset == Asset::Bitcoin && data.is_empty() {
                            eprintln!("Note: Bitcoin data on Yahoo Finance only available from Sept 2014 onwards");
                        }
                        
                        // Convert cents to dollars for certain futures
                        match asset {
                            Asset::Wheat | Asset::Corn | Asset::Soybeans | Asset::Coffee | 
                            Asset::Sugar | Asset::Cotton | Asset::Beef | Asset::Rice | Asset::Copper => {
                                for price in data.iter_mut() {
                                    price.price_usd /= 100.0;
                                }
                            },
                            _ => {}
                        }
                        all_data.insert(*asset, data);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch data for {} (ticker: {}): {}", 
                             asset_opt.map(|a| format!("{:?}", a)).unwrap_or("USD/CAD".to_string()), 
                             ticker, e);
                    
                    // If we hit rate limit, wait longer
                    if e.to_string().contains("429") {
                        println!("Rate limited! Waiting 10 seconds...");
                        sleep(Duration::from_secs(10)).await;
                    }
                }
            }
        }
        
        // Now add CAD prices to all assets
        if !usd_cad_rates.is_empty() {
            println!("Adding CAD prices using {} USD/CAD rates", usd_cad_rates.len());
            for (_asset, data) in all_data.iter_mut() {
                for price in data.iter_mut() {
                    // Find USD/CAD rate for this date
                    let cad_rate = usd_cad_rates.iter()
                        .find(|r| r.date == price.date)
                        .map(|r| r.price_usd)
                        .unwrap_or(1.35); // Default USD/CAD rate if not found
                    
                    price.price_cad = Some(price.price_usd * cad_rate);
                }
            }
        } else {
            println!("No USD/CAD rates available, using default 1.35");
            for (_asset, data) in all_data.iter_mut() {
                for price in data.iter_mut() {
                    price.price_cad = Some(price.price_usd * 1.35);
                }
            }
        }
        
        // Create consumer basket as weighted average of other assets
        if let Some(basket_data) = self.create_consumer_basket(&all_data, &usd_cad_rates) {
            all_data.insert(Asset::ConsumerBasket, basket_data);
        }
        
        println!("Finished fetching data. Got data for {} assets.", all_data.len());
        Ok(all_data)
    }
    
    async fn fetch_asset_data(&self, ticker: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<PriceData>> {
        // Use direct HTTP for all tickers since yahoo_finance_api is having issues
        self.fetch_ticker_direct(ticker, start, end).await
    }
    
    async fn fetch_asset_data_old(&self, ticker: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<PriceData>> {
        // Convert chrono DateTime to time OffsetDateTime
        let start_time = OffsetDateTime::from_unix_timestamp(start.timestamp())?;
        let end_time = OffsetDateTime::from_unix_timestamp(end.timestamp())?;
        
        let response = self.provider
            .get_quote_history(ticker, start_time, end_time)
            .await
            .map_err(|e| {
                if ticker == "CAD=X" {
                    eprintln!("CAD=X specific error: {:?}", e);
                }
                anyhow::anyhow!("Failed to fetch from Yahoo Finance: {}", e)
            })?;
            
        let quotes = response.quotes()
            .map_err(|e| anyhow::anyhow!("Failed to parse quotes: {}", e))?;
            
        if quotes.is_empty() {
            return Err(anyhow::anyhow!("No data available for this ticker"));
        }
            
        let mut data = Vec::new();
        
        for quote in quotes {
            let date = Utc.timestamp_opt(quote.timestamp as i64, 0)
                .single()
                .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?
                .date_naive();
                
            data.push(PriceData {
                date,
                price_usd: quote.close,
                price_cad: None, // Will be filled later
            });
        }
        
        Ok(data)
    }
    
    async fn fetch_ticker_direct(&self, ticker: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<PriceData>> {
        use reqwest::Client;
        use serde_json::Value;
        
        let client = Client::new();
        
        // Convert to Unix timestamps
        let period1 = start.timestamp();
        let period2 = end.timestamp();
        
        // Build URL with date range
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?period1={}&period2={}&interval=1d",
            ticker, period1, period2
        );
        
        let response = client.get(&url)
            .header("User-Agent", "Mozilla/5.0")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }
        
        let json: Value = response.json().await?;
        
        // Parse the Yahoo Finance response
        let result = &json["chart"]["result"][0];
        let timestamps = result["timestamp"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No timestamp data"))?;
        let quotes = &result["indicators"]["quote"][0];
        let closes = quotes["close"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No close price data"))?;
            
        let mut data = Vec::new();
        
        for (i, timestamp) in timestamps.iter().enumerate() {
            if let (Some(ts), Some(close)) = (timestamp.as_i64(), closes.get(i).and_then(|c| c.as_f64())) {
                let date = Utc.timestamp_opt(ts, 0)
                    .single()
                    .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?
                    .date_naive();
                    
                data.push(PriceData {
                    date,
                    price_usd: close,
                    price_cad: None, // Not needed for exchange rate itself
                });
            }
        }
        
        Ok(data)
    }
    
    fn create_consumer_basket(&self, all_data: &HashMap<Asset, Vec<PriceData>>, usd_cad_rates: &[PriceData]) -> Option<Vec<PriceData>> {
        use std::collections::HashMap as StdHashMap;
        use chrono::NaiveDate;
        
        // Define weights for consumer basket (roughly based on typical household spending)
        let weights = vec![
            (Asset::Oil, 0.15),        // Transportation fuel
            (Asset::NaturalGas, 0.05), // Heating/utilities
            (Asset::Wheat, 0.08),      // Bread/cereals
            (Asset::Corn, 0.05),       // Food products
            (Asset::Beef, 0.10),       // Meat
            (Asset::Coffee, 0.03),     // Beverages
            (Asset::Sugar, 0.02),      // Food additive
            (Asset::Cotton, 0.05),     // Clothing
            (Asset::Lumber, 0.07),     // Housing materials
            (Asset::Gold, 0.05),       // Jewelry/investment
            (Asset::Silver, 0.02),     // Electronics/jewelry
            (Asset::Copper, 0.03),     // Electronics/construction
            (Asset::Soybeans, 0.05),   // Food products
            (Asset::Rice, 0.05),       // Staple food
            // Remaining 20% represents services not captured by commodities
        ];
        
        // Create a map of dates to CAD rates for easy lookup
        let cad_rate_map: StdHashMap<NaiveDate, f64> = usd_cad_rates.iter()
            .map(|p| (p.date, p.price_usd))
            .collect();
        
        // Collect all unique dates from all assets
        let mut all_dates = std::collections::HashSet::new();
        for (asset, _) in &weights {
            if let Some(data) = all_data.get(asset) {
                for price in data {
                    all_dates.insert(price.date);
                }
            }
        }
        
        // Convert to sorted vector
        let mut dates: Vec<_> = all_dates.into_iter().collect();
        dates.sort();
        
        let mut basket_data = Vec::new();
        
        for date in dates {
            let mut weighted_sum = 0.0;
            let mut total_weight = 0.0;
            let mut weighted_cad_sum = 0.0;
            let mut cad_weight_total = 0.0;
            
            for (asset, weight) in &weights {
                if let Some(asset_data) = all_data.get(asset) {
                    // Find price for this date
                    if let Some(price_data) = asset_data.iter().find(|p| p.date == date) {
                        // Normalize prices to a base of 100
                        let normalized_price = match asset {
                            Asset::Gold => price_data.price_usd / 1800.0 * 100.0,
                            Asset::Silver => price_data.price_usd / 25.0 * 100.0,
                            Asset::Oil => price_data.price_usd / 75.0 * 100.0,
                            Asset::NaturalGas => price_data.price_usd / 3.5 * 100.0,
                            Asset::Copper => price_data.price_usd / 4.3 * 100.0,
                            Asset::Wheat => price_data.price_usd / 6.5 * 100.0,
                            Asset::Corn => price_data.price_usd / 5.0 * 100.0,
                            Asset::Soybeans => price_data.price_usd / 12.5 * 100.0,
                            Asset::Coffee => price_data.price_usd / 2.0 * 100.0,
                            Asset::Sugar => price_data.price_usd / 0.21 * 100.0,
                            Asset::Cotton => price_data.price_usd / 0.85 * 100.0,
                            Asset::Beef => price_data.price_usd / 1.4 * 100.0,
                            Asset::Rice => price_data.price_usd / 17.0 * 100.0,
                            Asset::Lumber => price_data.price_usd / 450.0 * 100.0,
                            _ => price_data.price_usd,
                        };
                        
                        weighted_sum += normalized_price * weight;
                        total_weight += weight;
                        
                        // Calculate CAD if available
                        if let Some(cad_price) = price_data.price_cad {
                            let normalized_cad_price = match asset {
                                Asset::Gold => cad_price / 2340.0 * 100.0,
                                Asset::Silver => cad_price / 32.5 * 100.0,
                                Asset::Oil => cad_price / 97.5 * 100.0,
                                Asset::NaturalGas => cad_price / 4.55 * 100.0,
                                Asset::Copper => cad_price / 5.59 * 100.0,
                                Asset::Wheat => cad_price / 8.45 * 100.0,
                                Asset::Corn => cad_price / 6.5 * 100.0,
                                Asset::Soybeans => cad_price / 16.25 * 100.0,
                                Asset::Coffee => cad_price / 2.6 * 100.0,
                                Asset::Sugar => cad_price / 0.273 * 100.0,
                                Asset::Cotton => cad_price / 1.105 * 100.0,
                                Asset::Beef => cad_price / 1.82 * 100.0,
                                Asset::Rice => cad_price / 22.1 * 100.0,
                                Asset::Lumber => cad_price / 585.0 * 100.0,
                                _ => cad_price,
                            };
                            
                            weighted_cad_sum += normalized_cad_price * weight;
                            cad_weight_total += weight;
                        }
                    }
                }
            }
            
            // Only add data point if we have enough components
            if total_weight >= 0.5 { // At least 50% of the basket components
                let usd_price = weighted_sum / total_weight * 100.0;
                
                let cad_price = if cad_weight_total >= 0.5 {
                    Some(weighted_cad_sum / cad_weight_total * 100.0)
                } else {
                    // If we don't have enough CAD data but have USD data and CAD rate, calculate it
                    cad_rate_map.get(&date).map(|rate| usd_price * rate)
                };
                
                basket_data.push(PriceData {
                    date,
                    price_usd: usd_price,
                    price_cad: cad_price,
                });
            }
        }
        
        if basket_data.is_empty() {
            println!("Warning: No consumer basket data could be calculated");
            None
        } else {
            println!("Created consumer basket with {} data points", basket_data.len());
            Some(basket_data)
        }
    }
}