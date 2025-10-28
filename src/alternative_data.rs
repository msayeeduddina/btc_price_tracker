use chrono::{DateTime, Utc, TimeZone, Duration, Datelike};
use std::collections::HashMap;
use anyhow::Result;
use crate::models::Asset;
use crate::data::PriceData;

pub struct AlternativeDataFetcher {
    client: reqwest::Client,
}

impl AlternativeDataFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_historical_data(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<HashMap<Asset, Vec<PriceData>>> {
        let mut all_data = HashMap::new();
        
        // For Bitcoin, use CoinGecko or alternative APIs
        if let Ok(btc_data) = self.fetch_bitcoin_data(start_date, end_date).await {
            all_data.insert(Asset::Bitcoin, btc_data);
        }
        
        // For commodities, we'll use static data for now due to rate limiting
        // In production, you'd want to use a proper API with authentication
        all_data.insert(Asset::Gold, self.generate_sample_commodity_data(Asset::Gold, 1800.0, 2100.0));
        all_data.insert(Asset::Silver, self.generate_sample_commodity_data(Asset::Silver, 22.0, 28.0));
        all_data.insert(Asset::Oil, self.generate_sample_commodity_data(Asset::Oil, 65.0, 85.0));
        all_data.insert(Asset::NaturalGas, self.generate_sample_commodity_data(Asset::NaturalGas, 2.5, 4.5));
        all_data.insert(Asset::Copper, self.generate_sample_commodity_data(Asset::Copper, 3.8, 4.8));
        all_data.insert(Asset::Wheat, self.generate_sample_commodity_data(Asset::Wheat, 5.0, 8.5));
        all_data.insert(Asset::Corn, self.generate_sample_commodity_data(Asset::Corn, 3.5, 7.0));
        all_data.insert(Asset::Soybeans, self.generate_sample_commodity_data(Asset::Soybeans, 10.0, 15.0));
        all_data.insert(Asset::Coffee, self.generate_sample_commodity_data(Asset::Coffee, 1.2, 2.8));
        all_data.insert(Asset::Sugar, self.generate_sample_commodity_data(Asset::Sugar, 0.18, 0.24));
        all_data.insert(Asset::Cotton, self.generate_sample_commodity_data(Asset::Cotton, 0.75, 0.95));
        all_data.insert(Asset::Beef, self.generate_sample_commodity_data(Asset::Beef, 1.0, 1.8));
        all_data.insert(Asset::Rice, self.generate_sample_commodity_data(Asset::Rice, 14.0, 20.0));
        all_data.insert(Asset::Lumber, self.generate_sample_commodity_data(Asset::Lumber, 300.0, 600.0));
        
        // Create consumer basket as weighted average
        if let Some(basket_data) = self.create_consumer_basket(&all_data) {
            all_data.insert(Asset::ConsumerBasket, basket_data);
        }
        
        Ok(all_data)
    }
    
    async fn fetch_bitcoin_data(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<PriceData>> {
        // Try CoinGecko API (free tier)
        let days = (end - start).num_days();
        let url = format!(
            "https://api.coingecko.com/api/v3/coins/bitcoin/market_chart?vs_currency=usd&days={}&interval=daily",
            days.min(365) // Free tier limited to 365 days
        );
        
        let response = self.client.get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch Bitcoin data: {}", response.status()));
        }
        
        let json: serde_json::Value = response.json().await?;
        let prices = json["prices"].as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;
            
        let mut data = Vec::new();
        for price_point in prices {
            if let (Some(timestamp), Some(price)) = (
                price_point[0].as_i64(),
                price_point[1].as_f64()
            ) {
                let date = Utc.timestamp_millis_opt(timestamp)
                    .single()
                    .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?
                    .date_naive();
                    
                data.push(PriceData {
                    date,
                    price_usd: price,
                    price_cad: Some(price * 1.32), // Fixed rate in fallback mode
                });
            }
        }
        
        Ok(data)
    }
    
    fn generate_sample_commodity_data(&self, asset: Asset, min_price: f64, max_price: f64) -> Vec<PriceData> {
        // Generate sample data for demonstration when Yahoo Finance is unavailable
        // In production, this would fetch from a proper commodity API
        let mut data = Vec::new();
        let start_date = Utc::now() - Duration::days(365 * 3);
        let mut current_price = (min_price + max_price) / 2.0;
        
        // Use a fixed USD/CAD rate since we don't have real exchange data in fallback mode
        const USD_CAD_RATE: f64 = 1.32; // Recent average
        
        for i in 0..1095 { // 3 years of daily data
            let date = (start_date + Duration::days(i)).date_naive();
            
            // Add some realistic volatility
            let change = (rand::thread_rng().gen::<f64>() - 0.5) * 0.02 * (max_price - min_price);
            current_price = (current_price + change).max(min_price).min(max_price);
            
            // Add seasonal patterns for agricultural commodities
            let seasonal_factor = match asset {
                Asset::Wheat | Asset::Corn => {
                    let day_of_year = date.ordinal() as f64;
                    1.0 + 0.1 * (2.0 * std::f64::consts::PI * day_of_year / 365.0).sin()
                },
                _ => 1.0,
            };
            
            data.push(PriceData {
                date,
                price_usd: current_price * seasonal_factor,
                price_cad: Some(current_price * seasonal_factor * USD_CAD_RATE),
            });
        }
        
        data
    }
    
    fn create_consumer_basket(&self, all_data: &HashMap<Asset, Vec<PriceData>>) -> Option<Vec<PriceData>> {
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
        
        // Get the minimum number of data points across all assets
        let min_len = weights.iter()
            .filter_map(|(asset, _)| all_data.get(asset).map(|d| d.len()))
            .min()?;
        
        let mut basket_data = Vec::with_capacity(min_len);
        
        for i in 0..min_len {
            let mut weighted_sum = 0.0;
            let mut total_weight = 0.0;
            let mut date = None;
            
            for (asset, weight) in &weights {
                if let Some(asset_data) = all_data.get(asset) {
                    if let Some(price_data) = asset_data.get(i) {
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
                        date = Some(price_data.date);
                    }
                }
            }
            
            if let Some(date) = date {
                if total_weight > 0.0 {
                    let usd_price = weighted_sum / total_weight * 100.0;
                    // Calculate weighted CAD price using actual CAD prices from components
                    let mut weighted_cad_sum = 0.0;
                    let mut cad_weight_total = 0.0;
                    
                    for (asset, weight) in &weights {
                        if let Some(asset_data) = all_data.get(asset) {
                            if let Some(price_data) = asset_data.get(i) {
                                if let Some(cad_price) = price_data.price_cad {
                                    // Normalize CAD prices same as USD
                                    let normalized_cad_price = match asset {
                                        Asset::Gold => cad_price / 2340.0 * 100.0,  // CAD price ~1800*1.3
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
                    
                    let cad_price = if cad_weight_total > 0.0 {
                        Some(weighted_cad_sum / cad_weight_total * 100.0)
                    } else {
                        None
                    };
                    
                    basket_data.push(PriceData {
                        date,
                        price_usd: usd_price,
                        price_cad: cad_price,
                    });
                }
            }
        }
        
        Some(basket_data)
    }
}

use rand::Rng;