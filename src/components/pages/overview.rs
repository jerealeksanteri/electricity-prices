use dioxus::prelude::*;

use crate::components::charts::forecast_line::ForecastLine;
use crate::components::charts::price_chart::PriceChart;
use crate::components::common::{Card, ErrorBanner, Eyebrow, RefreshingBanner, Skeleton, StatTile};
use crate::server::{
    entso::get_overview_data,
    FlowPoint, PricePoint, FI_AREA,
};

fn tier_class(eur_mwh: f64) -> &'static str {
    let c_kwh = eur_mwh / 10.0;
    if c_kwh < 5.0 {
        "text-tier-low"
    } else if c_kwh <= 15.0 {
        "text-tier-mid"
    } else {
        "text-tier-high"
    }
}

fn is_renewable(source: &str) -> bool {
    let s = source.to_lowercase();
    s.contains("wind")
        || s.contains("solar")
        || s.contains("hydro")
        || s.contains("biomass")
        || s.contains("geothermal")
        || s.contains("renewable")
}

#[component]
pub fn Overview() -> Element {
    let data = use_resource(|| get_overview_data(FI_AREA.to_string()));

    rsx! {
        div { class: "mb-6",
            Eyebrow { text: "Finnish bidding zone \u{00B7} 10YFI-1--------U".to_string() }
            h1 { class: "mt-1 font-display text-3xl font-bold tracking-tight text-ink", "System overview" }
        }

        match data() {
            Some(Ok(d)) => rsx! {
                HeroPrice { data: d.prices.clone() }

                div { class: "mt-4 grid grid-cols-2 gap-4 lg:grid-cols-4",
                    {
                        let total: f64 = d.generation.sources.iter().map(|s| s.value_mw).sum();
                        let renew: f64 = d.generation.sources.iter().filter(|s| is_renewable(&s.source_type)).map(|s| s.value_mw).sum();
                        let pct = if total > 0.0 { renew / total * 100.0 } else { 0.0 };
                        rsx! {
                            StatTile { label: "Generation now".to_string(), value: format!("{total:.0} MW"), hint: "all sources".to_string() }
                            StatTile { label: "Renewable share".to_string(), value: format!("{pct:.0}%"), accent: "text-aurora-green".to_string(), hint: format!("{renew:.0} MW renewable") }
                        }
                    }
                    {
                        let peak = peak_24h(&d.forecast);
                        rsx! { StatTile { label: "Peak load (24h)".to_string(), value: format!("{peak:.0} MW"), accent: "text-aurora-teal".to_string(), hint: "forecast".to_string() } }
                    }
                    {
                        let net = net_flow(&d.flows);
                        let (label, accent) = if net >= 0.0 { ("net import", "text-aurora-green") } else { ("net export", "text-aurora-violet") };
                        rsx! { StatTile { label: "Cross-border".to_string(), value: format!("{:.0} MW", net.abs()), accent: accent.to_string(), hint: label.to_string() } }
                    }
                }

                div { class: "mt-4 grid grid-cols-1 gap-4 lg:grid-cols-2",
                    Card { title: "Day-ahead prices \u{00B7} c/kWh".to_string(),
                        PriceChart { data: d.prices }
                    }
                    Card { title: "Load forecast \u{00B7} MW".to_string(),
                        ForecastLine { data: d.forecast }
                    }
                }
            },
            Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
            None => rsx! {
                RefreshingBanner {}
                Skeleton {}
                div { class: "mt-4 grid grid-cols-2 gap-4 lg:grid-cols-4",
                    Skeleton {} Skeleton {} Skeleton {} Skeleton {}
                }
                div { class: "mt-4 grid grid-cols-1 gap-4 lg:grid-cols-2",
                    Skeleton {} Skeleton {}
                }
            },
        }
    }
}

#[component]
fn HeroPrice(data: Vec<PricePoint>) -> Element {
    let now = chrono::Utc::now();
    let current = data
        .iter()
        .filter(|p| p.timestamp <= now)
        .max_by_key(|p| p.timestamp)
        .or_else(|| data.first());
    let avg = if data.is_empty() {
        0.0
    } else {
        data.iter().map(|p| p.price_eur_mwh).sum::<f64>() / data.len() as f64
    };
    match current {
        Some(p) => {
            let up = p.price_eur_mwh >= avg;
            let arrow = if up { "\u{25B2}" } else { "\u{25BC}" };
            let trend_class = if up { "text-tier-high" } else { "text-tier-low" };
            let c_kwh = p.price_eur_mwh / 10.0;
            let price_class = format!("readout text-6xl font-bold {}", tier_class(p.price_eur_mwh));
            rsx! {
                div { class: "panel animate-fade-up flex flex-col gap-6 p-6 sm:flex-row sm:items-end sm:justify-between",
                    div {
                        Eyebrow { text: "Current spot price".to_string() }
                        div { class: "mt-2 flex items-baseline gap-3",
                            span { class: "{price_class}",
                                {format!("{c_kwh:.2}")}
                            }
                            span { class: "readout text-lg text-muted", "c/kWh" }
                        }
                        div { class: "mt-1 readout text-sm text-faint", {format!("{:.1} \u{20AC}/MWh \u{00B7} as of {}", p.price_eur_mwh, p.timestamp.format("%H:%M UTC"))} }
                    }
                    div { class: "flex items-center gap-2 {trend_class}",
                        span { class: "readout text-lg", "{arrow}" }
                        span { class: "text-sm", {format!("{:+.2} c/kWh vs period avg", (p.price_eur_mwh - avg) / 10.0)} }
                    }
                }
            }
        }
        None => rsx! { ErrorBanner { msg: "no price data".to_string() } },
    }
}

fn peak_24h(f: &[crate::server::ForecastPoint]) -> f64 {
    let now = chrono::Utc::now();
    let horizon = now + chrono::Duration::hours(24);
    f.iter()
        .filter(|p| p.timestamp >= now && p.timestamp <= horizon)
        .map(|p| p.value_mw)
        .fold(0.0_f64, f64::max)
}

fn net_flow(flows: &[FlowPoint]) -> f64 {
    flows
        .iter()
        .map(|f| if f.to_area == "FI" { f.value_mw } else { -f.value_mw })
        .sum()
}
