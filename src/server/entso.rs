use dioxus::prelude::*;

use super::{ForecastPoint, FlowPoint, GenerationMix, PricePoint};

#[cfg(feature = "server")]
use chrono::{DateTime, Duration, Utc};
#[cfg(feature = "server")]
use entsoe::{
    Authorization, Domain, DocumentType, FlowMarketDocument, GlMarketDocument, PeriodTimestamp,
    ProcessType, PsrType, PublicationMarketDocument, RequestParameters,
};
#[cfg(feature = "server")]
use std::str::FromStr;
#[cfg(feature = "server")]
use std::sync::Arc;
#[cfg(feature = "server")]
use super::cache::EntsoeCache;
#[cfg(feature = "server")]
use super::{GenerationSource, ServerError};

// ---------- pure mapping helpers (server-only) ----------

#[cfg(feature = "server")]
fn parse_start(s: &str) -> Result<DateTime<Utc>, ServerError> {
    use chrono::NaiveDateTime;
    // ENTSO-E emits e.g. "2024-01-01T00:00Z" (no seconds) and sometimes full
    // RFC3339. Try RFC3339 first, then the minute-precision form (parsed as a
    // naive datetime and pinned to UTC, since the trailing Z denotes UTC).
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%MZ").map(|ndt| ndt.and_utc())
        })
        .map_err(|_| ServerError::BadTimestamp(s.to_string()))
}

#[cfg(feature = "server")]
fn resolution_minutes(res: &str) -> i64 {
    match res {
        "PT15M" => 15,
        "PT30M" => 30,
        "PT60M" | "PT1H" => 60,
        "P1D" => 24 * 60,
        _ => 60,
    }
}

#[cfg(feature = "server")]
fn point_time(start: DateTime<Utc>, position: u32, res_min: i64) -> DateTime<Utc> {
    start + Duration::minutes((position as i64 - 1) * res_min)
}

#[cfg(feature = "server")]
pub fn map_prices(doc: &PublicationMarketDocument) -> Result<Vec<PricePoint>, ServerError> {
    let mut out = Vec::new();
    for ts in &doc.time_series {
        let start = parse_start(&ts.period.time_interval.start)?;
        let res = resolution_minutes(&ts.period.resolution);
        for p in &ts.period.points {
            out.push(PricePoint {
                timestamp: point_time(start, p.position, res),
                price_eur_mwh: p.price_amount,
            });
        }
    }
    if out.is_empty() {
        return Err(ServerError::NoData("prices".into()));
    }
    out.sort_by_key(|p| p.timestamp);
    Ok(out)
}

#[cfg(feature = "server")]
pub fn map_generation(doc: &GlMarketDocument) -> Result<GenerationMix, ServerError> {
    let mut latest_ts: Option<DateTime<Utc>> = None;
    let mut sources: Vec<GenerationSource> = Vec::new();
    for ts in &doc.time_series {
        let start = parse_start(&ts.period.time_interval.start)?;
        let res = resolution_minutes(&ts.period.resolution);
        let Some(last) = ts.period.points.last() else { continue };
        let t = point_time(start, last.position, res);
        latest_ts = Some(latest_ts.map_or(t, |cur| cur.max(t)));
        let name = ts
            .mkt_psr_type
            .as_ref()
            .map(|m| PsrType::from_code(&m.psr_type).name().to_string())
            .unwrap_or_else(|| "Other".to_string());
        sources.push(GenerationSource { source_type: name, value_mw: last.quantity });
    }
    let timestamp = latest_ts.ok_or_else(|| ServerError::NoData("generation".into()))?;
    sources.sort_by(|a, b| b.value_mw.partial_cmp(&a.value_mw).unwrap_or(std::cmp::Ordering::Equal));
    Ok(GenerationMix { timestamp, sources })
}

#[cfg(feature = "server")]
pub fn map_forecast(doc: &GlMarketDocument) -> Result<Vec<ForecastPoint>, ServerError> {
    let mut out = Vec::new();
    for ts in &doc.time_series {
        let start = parse_start(&ts.period.time_interval.start)?;
        let res = resolution_minutes(&ts.period.resolution);
        for p in &ts.period.points {
            out.push(ForecastPoint { timestamp: point_time(start, p.position, res), value_mw: p.quantity });
        }
    }
    if out.is_empty() {
        return Err(ServerError::NoData("forecast".into()));
    }
    out.sort_by_key(|p| p.timestamp);
    Ok(out)
}

/// Most recent instantaneous physical flow (MW) for a single direction.
///
/// A11 returns hourly MW values; we want the latest available hour, not a sum
/// over the whole requested window (which would inflate the figure ~Nx).
#[cfg(feature = "server")]
pub fn latest_flow(doc: &FlowMarketDocument) -> f64 {
    doc.time_series
        .last()
        .and_then(|ts| ts.period.points.last())
        .map(|p| p.quantity)
        .unwrap_or(0.0)
}

// ---------- server-only request helpers ----------

#[cfg(feature = "server")]
fn window() -> (PeriodTimestamp, PeriodTimestamp) {
    let now = Utc::now();
    (
        PeriodTimestamp(now - Duration::hours(24)),
        PeriodTimestamp(now + Duration::hours(36)),
    )
}

#[cfg(feature = "server")]
async fn cache_ctx() -> Result<Arc<EntsoeCache>, ServerFnError> {
    use axum::Extension;
    let Extension(cache): Extension<Arc<EntsoeCache>> = extract().await?;
    Ok(cache)
}

#[cfg(feature = "server")]
fn domain(area: &str) -> Result<Domain, ServerFnError> {
    Domain::from_str(area).map_err(|_| ServerFnError::new(format!("unknown area {area}")))
}

#[cfg(feature = "server")]
fn to_sfe(e: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::new(e.to_string())
}

#[cfg(feature = "server")]
async fn sum_dir(
    cache: &EntsoeCache,
    out_d: Domain,
    in_d: Domain,
    start: &PeriodTimestamp,
    end: &PeriodTimestamp,
) -> f64 {
    let params = RequestParameters {
        document_type: DocumentType::A11,
        out_domain: Some(out_d),
        in_domain: Some(in_d),
        out_bidding_zone_domain: None,
        period_start: PeriodTimestamp(start.0),
        period_end: PeriodTimestamp(end.0),
        process_type: None,
        psr_type: None,
        authorization: Authorization::new(cache.token.clone()),
    };
    match cache.client.get_flows(&params).await {
        Ok(doc) => latest_flow(&doc),
        Err(_) => 0.0, // e.g. FI-RU "no matching data"
    }
}

// ---------- server functions (compiled on BOTH targets) ----------

#[server]
pub async fn get_spot_prices(area: String) -> Result<Vec<PricePoint>, ServerFnError> {
    let cache = cache_ctx().await?;
    if let Some(hit) = cache.get_prices(&area).await {
        return Ok(hit);
    }
    let d = domain(&area)?;
    let (start, end) = window();
    let params = RequestParameters {
        document_type: DocumentType::A44,
        out_domain: Some(d),
        in_domain: Some(d),
        out_bidding_zone_domain: None,
        period_start: start,
        period_end: end,
        process_type: None,
        psr_type: None,
        authorization: Authorization::new(cache.token.clone()),
    };
    let doc = cache.client.get_prices(&params).await.map_err(to_sfe)?;
    let mapped = map_prices(&doc).map_err(to_sfe)?;
    cache.put_prices(&area, mapped.clone()).await;
    Ok(mapped)
}

/// Day-ahead prices for an explicit `[start_ts, end_ts]` window (unix seconds).
/// Used by the Prices page's timeframe selector. Cached per area+range.
#[server]
pub async fn get_prices_range(
    area: String,
    start_ts: i64,
    end_ts: i64,
) -> Result<Vec<PricePoint>, ServerFnError> {
    let cache = cache_ctx().await?;
    let key = format!("{area}:{start_ts}:{end_ts}");
    if let Some(hit) = cache.get_prices(&key).await {
        return Ok(hit);
    }
    let d = domain(&area)?;
    let start = chrono::DateTime::<Utc>::from_timestamp(start_ts, 0)
        .ok_or_else(|| ServerFnError::new("invalid start timestamp"))?;
    let end = chrono::DateTime::<Utc>::from_timestamp(end_ts, 0)
        .ok_or_else(|| ServerFnError::new("invalid end timestamp"))?;
    if end <= start {
        return Err(ServerFnError::new("end must be after start"));
    }
    let params = RequestParameters {
        document_type: DocumentType::A44,
        out_domain: Some(d),
        in_domain: Some(d),
        out_bidding_zone_domain: None,
        period_start: PeriodTimestamp(start),
        period_end: PeriodTimestamp(end),
        process_type: None,
        psr_type: None,
        authorization: Authorization::new(cache.token.clone()),
    };
    let doc = cache.client.get_prices(&params).await.map_err(to_sfe)?;
    let mapped = map_prices(&doc).map_err(to_sfe)?;
    cache.put_prices(&key, mapped.clone()).await;
    Ok(mapped)
}

#[server]
pub async fn get_generation_mix(area: String) -> Result<GenerationMix, ServerFnError> {
    let cache = cache_ctx().await?;
    if let Some(hit) = cache.get_generation(&area).await {
        return Ok(hit);
    }
    let d = domain(&area)?;
    let (start, end) = window();
    let params = RequestParameters {
        document_type: DocumentType::A75,
        out_domain: None,
        in_domain: Some(d),
        out_bidding_zone_domain: None,
        period_start: start,
        period_end: end,
        process_type: Some(ProcessType::Realised),
        psr_type: None,
        authorization: Authorization::new(cache.token.clone()),
    };
    let doc = cache.client.get_generation(&params).await.map_err(to_sfe)?;
    let mapped = map_generation(&doc).map_err(to_sfe)?;
    cache.put_generation(&area, mapped.clone()).await;
    Ok(mapped)
}

#[server]
pub async fn get_consumption_forecast(area: String) -> Result<Vec<ForecastPoint>, ServerFnError> {
    let cache = cache_ctx().await?;
    if let Some(hit) = cache.get_forecast(&area).await {
        return Ok(hit);
    }
    let d = domain(&area)?;
    let (start, end) = window();
    let params = RequestParameters {
        document_type: DocumentType::A65,
        out_domain: None,
        in_domain: None,
        out_bidding_zone_domain: Some(d),
        period_start: start,
        period_end: end,
        process_type: Some(ProcessType::DayAhead),
        psr_type: None,
        authorization: Authorization::new(cache.token.clone()),
    };
    let doc = cache.client.get_load_forecast(&params).await.map_err(to_sfe)?;
    let mapped = map_forecast(&doc).map_err(to_sfe)?;
    cache.put_forecast(&area, mapped.clone()).await;
    Ok(mapped)
}

#[server]
pub async fn get_cross_border_flows(area: String) -> Result<Vec<FlowPoint>, ServerFnError> {
    let cache = cache_ctx().await?;
    if let Some(hit) = cache.get_flows(&area).await {
        return Ok(hit);
    }
    let fi = domain(&area)?;
    let borders = [("SE3", Domain::SE3), ("EE", Domain::EE), ("NO4", Domain::NO4), ("RU", Domain::RU)];
    let (start, end) = window();
    let now = Utc::now();
    let mut out = Vec::new();
    for (label, nb) in borders {
        let import = sum_dir(&cache, nb, fi, &start, &end).await;
        let export = sum_dir(&cache, fi, nb, &start, &end).await;
        let net = import - export; // >0 import into FI
        out.push(FlowPoint {
            from_area: if net >= 0.0 { label.to_string() } else { "FI".to_string() },
            to_area: if net >= 0.0 { "FI".to_string() } else { label.to_string() },
            value_mw: net.abs(),
            timestamp: now,
        });
    }
    cache.put_flows(&area, out.clone()).await;
    Ok(out)
}

// ---------- mapping tests (need the `server` feature for the mapping fns) ----------

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn resolution_minutes_known_values() {
        assert_eq!(resolution_minutes("PT60M"), 60);
        assert_eq!(resolution_minutes("PT15M"), 15);
        assert_eq!(resolution_minutes("weird"), 60);
    }

    #[test]
    fn point_time_advances_by_position() {
        let start = parse_start("2024-01-01T00:00Z").unwrap();
        assert_eq!(point_time(start, 1, 60), start);
        assert_eq!(point_time(start, 3, 60), start + Duration::hours(2));
    }

    #[test]
    fn map_prices_sample() {
        let xml = r#"<Publication_MarketDocument><mRID>a</mRID><revisionNumber>1</revisionNumber><type>A44</type><createdDateTime>x</createdDateTime><period.timeInterval><start>2024-01-01T00:00Z</start><end>2024-01-02T00:00Z</end></period.timeInterval><TimeSeries><mRID>1</mRID><auction.type>A01</auction.type><businessType>A62</businessType><in_Domain.mRID codingScheme="A01">10YFI-1--------U</in_Domain.mRID><out_Domain.mRID codingScheme="A01">10YFI-1--------U</out_Domain.mRID><currency_Unit.name>EUR</currency_Unit.name><price_Measure_Unit.name>MWH</price_Measure_Unit.name><curveType>A01</curveType><Period><timeInterval><start>2024-01-01T00:00Z</start><end>2024-01-01T02:00Z</end></timeInterval><resolution>PT60M</resolution><Point><position>1</position><price.amount>10.0</price.amount></Point><Point><position>2</position><price.amount>20.0</price.amount></Point></Period></TimeSeries></Publication_MarketDocument>"#;
        let doc: PublicationMarketDocument = quick_xml::de::from_str(xml).unwrap();
        let pts = map_prices(&doc).unwrap();
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[0].price_eur_mwh, 10.0);
        assert_eq!(pts[1].timestamp, pts[0].timestamp + Duration::hours(1));
    }
}