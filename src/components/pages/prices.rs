use dioxus::prelude::*;

use crate::server::{entso::get_spot_prices, FI_AREA, PricePoint};
use crate::components::common::{Skeleton, ErrorBanner, Card};
use crate::components::charts::price_chart::PriceChart;

#[component]
pub fn Prices() -> Element {
    let data = use_server_future(|| get_spot_prices(FI_AREA.to_string()))?;
    rsx! {
        h1 { class: "mb-4 text-2xl font-bold", "Day-ahead prices" }
        SuspenseBoundary {
            fallback: |_ctx: SuspenseContext| rsx! { Skeleton {} },
            match data() {
                Some(Ok(d)) => rsx! {
                    PriceStats { data: d.clone() }
                    Card { title: "Next 24h (EUR/MWh)".to_string(), PriceChart { data: d } }
                },
                Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                None => rsx! { Skeleton {} },
            }
        }
    }
}

#[component]
fn PriceStats(data: Vec<PricePoint>) -> Element {
    let (min, max, avg) = stats(&data);
    rsx! {
        div { class: "mb-4 grid grid-cols-3 gap-4",
            StatCard { label: "Min".to_string(), value: format!("{min:.1} EUR/MWh") }
            StatCard { label: "Avg".to_string(), value: format!("{avg:.1} EUR/MWh") }
            StatCard { label: "Max".to_string(), value: format!("{max:.1} EUR/MWh") }
        }
    }
}

#[component]
fn StatCard(label: String, value: String) -> Element {
    rsx! {
        div { class: "rounded-lg border border-gray-700 bg-gray-800 p-4",
            div { class: "text-xs uppercase text-gray-400", "{label}" }
            div { class: "text-xl font-semibold", "{value}" }
        }
    }
}

fn stats(data: &[PricePoint]) -> (f64, f64, f64) {
    if data.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let mut min = f64::MAX;
    let mut max = f64::MIN;
    let mut sum = 0.0;
    for p in data {
        min = min.min(p.price_eur_mwh);
        max = max.max(p.price_eur_mwh);
        sum += p.price_eur_mwh;
    }
    (min, max, sum / data.len() as f64)
}
