use dioxus::prelude::*;

use crate::server::{
    entso::{get_spot_prices, get_generation_mix, get_consumption_forecast},
    FI_AREA,
    PricePoint,
};
use crate::components::common::{Skeleton, ErrorBanner, Card};
use crate::components::charts::price_chart::PriceChart;

#[component]
pub fn Overview() -> Element {
    let prices = use_server_future(|| get_spot_prices(FI_AREA.to_string()))?;
    let gen = use_server_future(|| get_generation_mix(FI_AREA.to_string()))?;
    let fc = use_server_future(|| get_consumption_forecast(FI_AREA.to_string()))?;

    rsx! {
        h1 { class: "mb-4 text-2xl font-bold", "Overview" }
        SuspenseBoundary {
            fallback: |_ctx: SuspenseContext| rsx! { Skeleton {} },
            div { class: "grid grid-cols-1 gap-4 md:grid-cols-2",
                Card { title: "Current spot price".to_string(),
                    match prices() {
                        Some(Ok(d)) => rsx! { CurrentPrice { data: d } },
                        Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                        None => rsx! { Skeleton {} },
                    }
                }
                Card { title: "Generation total".to_string(),
                    match gen() {
                        Some(Ok(d)) => rsx! {
                            div { class: "text-3xl font-bold",
                                {format!("{:.0} MW", d.sources.iter().map(|s| s.value_mw).sum::<f64>())}
                            }
                        },
                        Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                        None => rsx! { Skeleton {} },
                    }
                }
                Card { title: "Peak forecast (next 24h)".to_string(),
                    match fc() {
                        Some(Ok(d)) => rsx! {
                            div { class: "text-3xl font-bold",
                                {format!("{:.0} MW", d.iter().map(|p| p.value_mw).fold(0.0_f64, f64::max))}
                            }
                        },
                        Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                        None => rsx! { Skeleton {} },
                    }
                }
                Card { title: "Today's prices".to_string(),
                    match prices() {
                        Some(Ok(d)) => rsx! { PriceChart { data: d } },
                        Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                        None => rsx! { Skeleton {} },
                    }
                }
            }
        }
    }
}

#[component]
fn CurrentPrice(data: Vec<PricePoint>) -> Element {
    let now = chrono::Utc::now();
    let current = data
        .iter()
        .min_by_key(|p| (p.timestamp - now).num_seconds().abs());
    let yest_avg = if data.is_empty() {
        0.0
    } else {
        data.iter().map(|p| p.price_eur_mwh).sum::<f64>() / data.len() as f64
    };
    match current {
        Some(p) => {
            let up = p.price_eur_mwh >= yest_avg;
            rsx! {
                div { class: "text-4xl font-bold", {format!("{:.1} EUR/MWh", p.price_eur_mwh)} }
                div {
                    class: if up { "text-red-400" } else { "text-emerald-400" },
                    if up { "above avg" } else { "below avg" }
                }
            }
        }
        None => rsx! { ErrorBanner { msg: "no price data".to_string() } },
    }
}
