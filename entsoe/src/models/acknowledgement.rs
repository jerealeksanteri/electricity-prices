use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "Acknowledgement_MarketDocument")]
pub struct AcknowledgementMarketDocument {
    #[serde(rename = "mRID")]
    pub mrid: String,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: String,
    #[serde(rename = "Reason")]
    pub reason: Reason,
}

#[derive(Debug, Deserialize)]
pub struct Reason {
    pub code: String,
    pub text: String,
}

