//! Server-side data layer.
//!
//! Shared domain types and the `#[server]` functions compile on BOTH the web
//! client and the native server. The `entsoe` client, cache, and XML mapping
//! are server-only (`#[cfg(feature = "server")]`).

pub mod entso;
#[cfg(feature = "server")]
pub mod cache;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub price_eur_mwh: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GenerationSource {
    pub source_type: String,
    pub value_mw: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GenerationMix {
    pub timestamp: DateTime<Utc>,
    pub sources: Vec<GenerationSource>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ForecastPoint {
    pub timestamp: DateTime<Utc>,
    pub value_mw: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FlowPoint {
    pub from_area: String,
    pub to_area: String,
    pub value_mw: f64,
    pub timestamp: DateTime<Utc>,
}

/// Default Finnish bidding zone EIC.
pub const FI_AREA: &str = "10YFI-1--------U";

/// All data needed by the Overview page, fetched in a single server round-trip.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OverviewData {
    pub prices: Vec<PricePoint>,
    pub generation: GenerationMix,
    pub forecast: Vec<ForecastPoint>,
    pub flows: Vec<FlowPoint>,
}

/// All data needed by the Grid page, fetched in a single server round-trip.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GridData {
    pub generation: GenerationMix,
    pub forecast: Vec<ForecastPoint>,
    pub flows: Vec<FlowPoint>,
}

#[cfg(feature = "server")]
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("ENTSO-E client error: {0}")]
    Entsoe(#[from] entsoe::EntsoeError),
    #[error("unknown area code: {0}")]
    UnknownArea(String),
    #[error("no data returned for {0}")]
    NoData(String),
    #[error("malformed timestamp: {0}")]
    BadTimestamp(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_point_round_trips() {
        let p = PricePoint { timestamp: "2024-01-01T00:00:00Z".parse().unwrap(), price_eur_mwh: 42.5 };
        let json = serde_json::to_string(&p).unwrap();
        let back: PricePoint = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }
}