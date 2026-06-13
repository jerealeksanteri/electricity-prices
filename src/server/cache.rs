use std::sync::Arc;
use std::time::Duration;

use chrono::{Datelike, TimeZone, Utc};
use chrono_tz::Europe::Helsinki;
use moka::future::Cache;
use moka::Expiry;

use entsoe::{Authorization, Entsoe};

use super::{ForecastPoint, FlowPoint, GenerationMix, PricePoint};

/// Local publish hour/minute for day-ahead prices (Europe/Helsinki).
const PUBLISH_HOUR: u32 = 13;
const PUBLISH_MIN: u32 = 15;

/// Duration from now until the next 13:15 Europe/Helsinki.
pub fn next_price_publish() -> Duration {
    let now = Utc::now().with_timezone(&Helsinki);
    let today = now.date_naive();
    let today_target = Helsinki
        .with_ymd_and_hms(today.year(), today.month(), today.day(), PUBLISH_HOUR, PUBLISH_MIN, 0)
        .single();
    let target = match today_target {
        Some(t) if t > now => t,
        _ => {
            let tomorrow = (now + chrono::Duration::days(1)).date_naive();
            Helsinki
                .with_ymd_and_hms(tomorrow.year(), tomorrow.month(), tomorrow.day(), PUBLISH_HOUR, PUBLISH_MIN, 0)
                .single()
                .expect("valid Helsinki datetime")
        }
    };
    (target.with_timezone(&Utc) - now.with_timezone(&Utc))
        .to_std()
        .unwrap_or(Duration::from_secs(60))
}

/// Per-entry expiry pinning price entries to the next publish time.
struct PriceExpiry;
impl<K, V> Expiry<K, V> for PriceExpiry {
    fn expire_after_create(&self, _key: &K, _value: &V, _now: std::time::Instant) -> Option<Duration> {
        Some(next_price_publish())
    }
}

/// In-memory cache fronting the ENTSO-E client. One typed cache per data type
/// (values differ), keyed by `"{data_type}:{area}"`.
pub struct EntsoeCache {
    pub client: Entsoe,
    pub token: String,
    spot_prices: Cache<String, Vec<PricePoint>>,
    generation: Cache<String, GenerationMix>,
    forecast: Cache<String, Vec<ForecastPoint>>,
    flows: Cache<String, Vec<FlowPoint>>,
}

impl EntsoeCache {
    pub fn new(token: String) -> Self {
        Self {
            client: Entsoe::new(Authorization::new(token.clone())),
            token,
            spot_prices: Cache::builder().expire_after(PriceExpiry).build(),
            generation: Cache::builder().time_to_live(Duration::from_secs(10 * 60)).build(),
            forecast: Cache::builder().time_to_live(Duration::from_secs(30 * 60)).build(),
            flows: Cache::builder().time_to_live(Duration::from_secs(10 * 60)).build(),
        }
    }

    pub async fn get_prices(&self, area: &str) -> Option<Vec<PricePoint>> {
        self.spot_prices.get(&format!("spot_prices:{area}")).await
    }
    pub async fn put_prices(&self, area: &str, v: Vec<PricePoint>) {
        self.spot_prices.insert(format!("spot_prices:{area}"), v).await;
    }
    pub async fn get_generation(&self, area: &str) -> Option<GenerationMix> {
        self.generation.get(&format!("generation:{area}")).await
    }
    pub async fn put_generation(&self, area: &str, v: GenerationMix) {
        self.generation.insert(format!("generation:{area}"), v).await;
    }
    pub async fn get_forecast(&self, area: &str) -> Option<Vec<ForecastPoint>> {
        self.forecast.get(&format!("forecast:{area}")).await
    }
    pub async fn put_forecast(&self, area: &str, v: Vec<ForecastPoint>) {
        self.forecast.insert(format!("forecast:{area}"), v).await;
    }
    pub async fn get_flows(&self, area: &str) -> Option<Vec<FlowPoint>> {
        self.flows.get(&format!("flows:{area}")).await
    }
    pub async fn put_flows(&self, area: &str, v: Vec<FlowPoint>) {
        self.flows.insert(format!("flows:{area}"), v).await;
    }
}

pub type SharedCache = Arc<EntsoeCache>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_publish_is_within_24h_and_positive() {
        let d = next_price_publish();
        assert!(d.as_secs() > 0);
        assert!(d.as_secs() <= 24 * 3600 + 60);
    }

    #[tokio::test]
    async fn cache_round_trips_prices() {
        let cache = EntsoeCache::new("tok".into());
        assert!(cache.get_prices("FI").await.is_none());
        cache.put_prices("FI", vec![PricePoint { timestamp: Utc::now(), price_eur_mwh: 1.0 }]).await;
        assert_eq!(cache.get_prices("FI").await.unwrap().len(), 1);
    }
}