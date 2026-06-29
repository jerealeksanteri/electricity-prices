use serde::Deserialize;

/// GL_MarketDocument is used by A75 (actual generation per type) and
/// A65 (load forecast). Quantities are MW. Generation series carry an
/// `MktPSRType.psrType`; load series do not.
#[derive(Debug, Deserialize)]
#[serde(rename = "GL_MarketDocument")]
pub struct GlMarketDocument {
    #[serde(rename = "mRID")]
    pub mrid: String,
    #[serde(rename = "type")]
    pub doc_type: String,
    #[serde(rename = "TimeSeries", default)]
    pub time_series: Vec<GlTimeSeries>,
}

#[derive(Debug, Deserialize)]
pub struct GlTimeSeries {
    #[serde(rename = "mRID")]
    pub mrid: String,
    #[serde(rename = "businessType")]
    pub business_type: String,
    #[serde(rename = "MktPSRType", default)]
    pub mkt_psr_type: Option<MktPsrType>,
    /// ENTSO-E can emit more than one `<Period>` per `<TimeSeries>` (e.g. the
    /// PT60M→PT15M market-time-unit transition, or DST-spanning days), so this
    /// must be a collection rather than a single value.
    #[serde(rename = "Period", default)]
    pub periods: Vec<GlPeriod>,
}

#[derive(Debug, Deserialize)]
pub struct MktPsrType {
    #[serde(rename = "psrType")]
    pub psr_type: String,
}

#[derive(Debug, Deserialize)]
pub struct GlPeriod {
    #[serde(rename = "timeInterval")]
    pub time_interval: super::prices::TimeInterval,
    pub resolution: String,
    #[serde(rename = "Point", default)]
    pub points: Vec<GlPoint>,
}

#[derive(Debug, Deserialize)]
pub struct GlPoint {
    pub position: u32,
    pub quantity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_GEN: &str = r#"
    <GL_MarketDocument>
      <mRID>abc</mRID>
      <type>A75</type>
      <TimeSeries>
        <mRID>1</mRID>
        <businessType>A01</businessType>
        <MktPSRType><psrType>B14</psrType></MktPSRType>
        <Period>
          <timeInterval><start>2024-01-01T00:00Z</start><end>2024-01-01T01:00Z</end></timeInterval>
          <resolution>PT60M</resolution>
          <Point><position>1</position><quantity>2400.0</quantity></Point>
        </Period>
      </TimeSeries>
    </GL_MarketDocument>"#;

    #[test]
    fn parses_generation_doc() {
        let doc: GlMarketDocument = quick_xml::de::from_str(SAMPLE_GEN).unwrap();
        assert_eq!(doc.doc_type, "A75");
        assert_eq!(doc.time_series.len(), 1);
        let ts = &doc.time_series[0];
        assert_eq!(ts.mkt_psr_type.as_ref().unwrap().psr_type, "B14");
        assert_eq!(ts.periods[0].points[0].quantity, 2400.0);
    }
}
