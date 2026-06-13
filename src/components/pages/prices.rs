use dioxus::prelude::*;

use crate::components::charts::price_chart::PriceChart;
use crate::components::common::{Card, ErrorBanner, Eyebrow, Skeleton, StatTile};
use crate::server::{entso::get_prices_range, PricePoint, FI_AREA};

const PRESETS: [&str; 4] = ["Today", "Tomorrow", "7 days", "30 days"];

fn today() -> chrono::NaiveDate {
    chrono::Utc::now().date_naive()
}

fn day_start_ts(d: chrono::NaiveDate) -> i64 {
    d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp()
}

fn default_start() -> i64 {
    day_start_ts(today())
}
fn default_end() -> i64 {
    day_start_ts(today() + chrono::Duration::days(1))
}

fn preset_range(name: &str) -> (i64, i64) {
    let t = today();
    let d = chrono::Duration::days(1);
    match name {
        "Today" => (day_start_ts(t), day_start_ts(t + d)),
        "Tomorrow" => (day_start_ts(t + d), day_start_ts(t + d * 2)),
        "7 days" => (day_start_ts(t - d * 6), day_start_ts(t + d)),
        "30 days" => (day_start_ts(t - d * 29), day_start_ts(t + d)),
        _ => (default_start(), default_end()),
    }
}

fn ts_to_date_str(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

fn parse_day(s: &str) -> Option<i64> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .ok()
        .map(day_start_ts)
}

#[component]
pub fn Prices() -> Element {
    let mut start = use_signal(default_start);
    let mut end = use_signal(default_end);
    let mut preset = use_signal(|| "Today".to_string());

    let data = use_server_future(move || get_prices_range(FI_AREA.to_string(), start(), end()))?;

    let start_date = ts_to_date_str(start());
    let end_date = ts_to_date_str(end() - 86_400); // show inclusive last day

    rsx! {
        div { class: "mb-6",
            Eyebrow { text: "Day-ahead market \u{00B7} ENTSO-E A44".to_string() }
            h1 { class: "mt-1 font-display text-3xl font-bold tracking-tight text-ink", "Electricity prices" }
        }

        // Timeframe controls
        div { class: "panel animate-fade-up mb-4 flex flex-col gap-4 p-5 lg:flex-row lg:items-end lg:justify-between",
            div {
                Eyebrow { text: "Quick range".to_string() }
                div { class: "mt-2 flex flex-wrap gap-2",
                    for name in PRESETS {
                        button {
                            class: if preset() == name { "pill pill-active" } else { "pill" },
                            onclick: move |_| {
                                let (s, e) = preset_range(name);
                                start.set(s);
                                end.set(e);
                                preset.set(name.to_string());
                            },
                            "{name}"
                        }
                    }
                }
            }
            div { class: "flex items-end gap-3",
                label { class: "flex flex-col gap-1",
                    span { class: "eyebrow", "From" }
                    input {
                        r#type: "date",
                        value: "{start_date}",
                        class: "rounded-lg border border-line bg-surface px-3 py-1.5 text-sm text-ink outline-none focus:border-aurora-green/50",
                        oninput: move |e| {
                            if let Some(ts) = parse_day(&e.value()) {
                                start.set(ts);
                                preset.set("Custom".to_string());
                            }
                        },
                    }
                }
                label { class: "flex flex-col gap-1",
                    span { class: "eyebrow", "To" }
                    input {
                        r#type: "date",
                        value: "{end_date}",
                        class: "rounded-lg border border-line bg-surface px-3 py-1.5 text-sm text-ink outline-none focus:border-aurora-green/50",
                        oninput: move |e| {
                            if let Some(ts) = parse_day(&e.value()) {
                                end.set(ts + 86_400); // inclusive end day
                                preset.set("Custom".to_string());
                            }
                        },
                    }
                }
            }
        }

        match data() {
            Some(Ok(d)) if !d.is_empty() => rsx! {
                PriceStats { data: d.clone() }
                Card { title: "Price curve \u{00B7} c/kWh".to_string(), PriceChart { data: d } }
            },
            Some(Ok(_)) => rsx! {
                div { class: "panel p-8 text-center text-muted",
                    "No price data published for this range yet."
                }
            },
            Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
            None => rsx! { Skeleton {} },
        }
    }
}

#[component]
fn PriceStats(data: Vec<PricePoint>) -> Element {
    let n = data.len().max(1) as f64;
    let avg = data.iter().map(|p| p.price_eur_mwh).sum::<f64>() / n;
    let min = data
        .iter()
        .min_by(|a, b| a.price_eur_mwh.total_cmp(&b.price_eur_mwh));
    let max = data
        .iter()
        .max_by(|a, b| a.price_eur_mwh.total_cmp(&b.price_eur_mwh));
    let min_hint = min
        .map(|p| format!("at {}", p.timestamp.format("%a %H:%M")))
        .unwrap_or_default();
    let max_hint = max
        .map(|p| format!("at {}", p.timestamp.format("%a %H:%M")))
        .unwrap_or_default();
    rsx! {
        div { class: "mb-4 grid grid-cols-2 gap-4 lg:grid-cols-4",
            StatTile {
                label: "Cheapest".to_string(),
                value: format!("{:.2}", min.map(|p| p.price_eur_mwh).unwrap_or(0.0) / 10.0),
                accent: "text-tier-low".to_string(),
                hint: min_hint,
            }
            StatTile { label: "Average".to_string(), value: format!("{:.2}", avg / 10.0), hint: "c/kWh".to_string() }
            StatTile {
                label: "Most expensive".to_string(),
                value: format!("{:.2}", max.map(|p| p.price_eur_mwh).unwrap_or(0.0) / 10.0),
                accent: "text-tier-high".to_string(),
                hint: max_hint,
            }
            StatTile { label: "Hours".to_string(), value: format!("{}", data.len()), hint: "data points".to_string() }
        }
    }
}
