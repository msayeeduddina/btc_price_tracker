use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPrice {
    pub asset: Asset,
    pub price_usd: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Asset {
    Bitcoin,
    Gold,
    Wheat,
    Corn,
    Beef,
    Coffee,
    Rice,
    Oil,
    NaturalGas,
    Copper,
    Silver,
    Soybeans,
    Sugar,
    Cotton,
    Lumber,
    ConsumerBasket, // Blended index
}

impl Asset {
    pub fn name(&self) -> &'static str {
        match self {
            Asset::Bitcoin => "Bitcoin",
            Asset::Gold => "Gold (per oz)",
            Asset::Wheat => "Wheat (per bushel)",
            Asset::Corn => "Corn (per bushel)",
            Asset::Beef => "Beef (per lb)",
            Asset::Coffee => "Coffee (per lb)",
            Asset::Rice => "Rice (per cwt)",
            Asset::Oil => "Crude Oil (per barrel)",
            Asset::NaturalGas => "Natural Gas (per MMBtu)",
            Asset::Copper => "Copper (per lb)",
            Asset::Silver => "Silver (per oz)",
            Asset::Soybeans => "Soybeans (per bushel)",
            Asset::Sugar => "Sugar (per lb)",
            Asset::Cotton => "Cotton (per lb)",
            Asset::Lumber => "Lumber (per 1000 bd ft)",
            Asset::ConsumerBasket => "Consumer Basket (Blended)",
        }
    }
    
    pub fn base_name(&self) -> &'static str {
        match self {
            Asset::Bitcoin => "Bitcoin",
            Asset::Gold => "Gold",
            Asset::Wheat => "Wheat",
            Asset::Corn => "Corn",
            Asset::Beef => "Beef",
            Asset::Coffee => "Coffee",
            Asset::Rice => "Rice",
            Asset::Oil => "Oil",
            Asset::NaturalGas => "Natural Gas",
            Asset::Copper => "Copper",
            Asset::Silver => "Silver",
            Asset::Soybeans => "Soybeans",
            Asset::Sugar => "Sugar",
            Asset::Cotton => "Cotton",
            Asset::Lumber => "Lumber",
            Asset::ConsumerBasket => "Consumer Basket",
        }
    }
    
    pub fn unit(&self) -> &'static str {
        match self {
            Asset::Bitcoin => "BTC",
            Asset::Gold => "oz",
            Asset::Wheat => "bushel",
            Asset::Corn => "bushel",
            Asset::Beef => "lb",
            Asset::Coffee => "lb",
            Asset::Rice => "cwt",
            Asset::Oil => "barrel",
            Asset::NaturalGas => "MMBtu",
            Asset::Copper => "lb",
            Asset::Silver => "oz",
            Asset::Soybeans => "bushel",
            Asset::Sugar => "lb",
            Asset::Cotton => "lb",
            Asset::Lumber => "1000 bd ft",
            Asset::ConsumerBasket => "basket",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PriceComparison {
    pub base_asset: Asset,
    pub target_asset: Asset,
    pub ratio: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepresentationMode {
    UnitsPerCurrency,  // How many units of asset per 1 BTC/Dollar
    PricePerUnit,      // Price in BTC/Dollars per 1 unit of asset
}

