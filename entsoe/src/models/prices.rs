use chrono::{
    DateTime,
    Utc
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "Publication_MarketDocument")]
pub struct PublicationMarketDocument {
    #[serde(rename = "mRID")]
    pub mrid: String,
    #[serde(rename = "revisionNumber")]
    pub revision_number: u32,
    #[serde(rename = "type")]
    pub doc_type: String,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: String,
    #[serde(rename = "period.timeInterval")]
    pub period_time_interval: TimeInterval,
    #[serde(rename = "TimeSeries")]
    pub time_series: Vec<TimeSeries>,
}

#[derive(Debug, Deserialize)]
pub struct TimeInterval {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Deserialize)]
pub struct TimeSeries {
    #[serde(rename = "mRID")]
    pub mrid: String,
    #[serde(rename = "auction.type")]
    pub auction_type: String,
    #[serde(rename = "businessType")]
    pub business_type: String,
    #[serde(rename = "in_Domain.mRID")]
    pub in_domain_mrid: DomainMrid,
    #[serde(rename = "out_Domain.mRID")]
    pub out_domain_mrid: DomainMrid,
    /// Present in some responses (e.g. A01 day-ahead auction)
    #[serde(rename = "contract_MarketAgreement.type", default)]
    pub contract_market_agreement_type: Option<String>,
    #[serde(rename = "currency_Unit.name")]
    pub currency_unit: String,
    #[serde(rename = "price_Measure_Unit.name")]
    pub price_measure_unit: String,
    #[serde(rename = "curveType")]
    pub curve_type: String,
    #[serde(rename = "Period")]
    pub period: Period,
}

#[derive(Debug, Deserialize)]
pub struct DomainMrid {
    /// XML attribute — quick_xml requires the `@` prefix for attributes
    #[serde(rename = "@codingScheme")]
    pub coding_scheme: String,
    #[serde(rename = "$text")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Period {
    #[serde(rename = "timeInterval")]
    pub time_interval: TimeInterval,
    pub resolution: String,
    #[serde(rename = "Point")]
    pub points: Vec<Point>,
}

#[derive(Debug, Deserialize)]
pub struct Point {
    pub position: u32,
    #[serde(rename = "price.amount")]
    pub price_amount: f64,
}