use serde::Deserialize;
use super::prices::{TimeInterval, DomainMrid};

/// A11 physical-flow Publication_MarketDocument. Like prices but Points carry
/// `quantity` (MW) instead of `price.amount`.
#[derive(Debug, Deserialize)]
#[serde(rename = "Publication_MarketDocument")]
pub struct FlowMarketDocument {
    #[serde(rename = "mRID")]
    pub mrid: String,
    #[serde(rename = "TimeSeries", default)]
    pub time_series: Vec<FlowTimeSeries>,
}

#[derive(Debug, Deserialize)]
pub struct FlowTimeSeries {
    #[serde(rename = "in_Domain.mRID")]
    pub in_domain_mrid: DomainMrid,
    #[serde(rename = "out_Domain.mRID")]
    pub out_domain_mrid: DomainMrid,
    #[serde(rename = "Period")]
    pub period: FlowPeriod,
}

#[derive(Debug, Deserialize)]
pub struct FlowPeriod {
    #[serde(rename = "timeInterval")]
    pub time_interval: TimeInterval,
    pub resolution: String,
    #[serde(rename = "Point", default)]
    pub points: Vec<FlowPoint>,
}

#[derive(Debug, Deserialize)]
pub struct FlowPoint {
    pub position: u32,
    pub quantity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
    <Publication_MarketDocument>
      <mRID>x</mRID>
      <TimeSeries>
        <in_Domain.mRID codingScheme="A01">10YFI-1--------U</in_Domain.mRID>
        <out_Domain.mRID codingScheme="A01">10Y1001A1001A46L</out_Domain.mRID>
        <Period>
          <timeInterval><start>2024-01-01T00:00Z</start><end>2024-01-01T01:00Z</end></timeInterval>
          <resolution>PT60M</resolution>
          <Point><position>1</position><quantity>500.0</quantity></Point>
        </Period>
      </TimeSeries>
    </Publication_MarketDocument>"#;

    #[test]
    fn parses_flow_doc() {
        let doc: FlowMarketDocument = quick_xml::de::from_str(SAMPLE).unwrap();
        assert_eq!(doc.time_series.len(), 1);
        assert_eq!(doc.time_series[0].period.points[0].quantity, 500.0);
        assert_eq!(doc.time_series[0].in_domain_mrid.value, "10YFI-1--------U");
    }
}