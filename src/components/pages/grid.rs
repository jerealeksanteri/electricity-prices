use dioxus::prelude::*;

use crate::components::charts::flow_map::FlowChart;
use crate::components::charts::forecast_line::ForecastLine;
use crate::components::charts::generation_pie::GenerationPie;
use crate::components::common::{Card, ErrorBanner, Eyebrow, Skeleton};
use crate::server::{
    entso::{get_consumption_forecast, get_cross_border_flows, get_generation_mix},
    FlowPoint, GenerationMix, FI_AREA,
};

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
pub fn Grid() -> Element {
    let gen_mix = use_server_future(|| get_generation_mix(FI_AREA.to_string()))?;
    let fc = use_server_future(|| get_consumption_forecast(FI_AREA.to_string()))?;
    let flows = use_server_future(|| get_cross_border_flows(FI_AREA.to_string()))?;

    rsx! {
        div { class: "mb-6",
            Eyebrow { text: "Generation \u{00B7} load \u{00B7} interconnectors".to_string() }
            h1 { class: "mt-1 font-display text-3xl font-bold tracking-tight text-ink", "Grid & generation" }
        }

        div { class: "grid grid-cols-1 gap-4 lg:grid-cols-5",
            div { class: "lg:col-span-3",
                Card { title: "Generation mix \u{00B7} MW".to_string(),
                    match gen_mix() {
                        Some(Ok(g)) => rsx! { GenerationPie { data: g } },
                        Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                        None => rsx! { Skeleton {} },
                    }
                }
            }
            div { class: "lg:col-span-2",
                Card { title: "By source".to_string(),
                    match gen_mix() {
                        Some(Ok(g)) => rsx! { SourceList { data: g } },
                        Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                        None => rsx! { Skeleton {} },
                    }
                }
            }
        }

        div { class: "mt-4 grid grid-cols-1 gap-4 lg:grid-cols-2",
            Card { title: "Load forecast \u{00B7} MW".to_string(),
                match fc() {
                    Some(Ok(f)) => rsx! { ForecastLine { data: f } },
                    Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                    None => rsx! { Skeleton {} },
                }
            }
            Card { title: "Cross-border flows \u{00B7} MW".to_string(),
                match flows() {
                    Some(Ok(fl)) => rsx! { FlowChart { data: fl.clone() } FlowTable { data: fl } },
                    Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                    None => rsx! { Skeleton {} },
                }
            }
        }
    }
}

#[component]
fn SourceList(data: GenerationMix) -> Element {
    let max = data
        .sources
        .iter()
        .map(|s| s.value_mw)
        .fold(0.0_f64, f64::max)
        .max(1.0);
    rsx! {
        div { class: "space-y-3",
            for s in data.sources {
                {
                    let pct = (s.value_mw / max * 100.0).clamp(0.0, 100.0);
                    let bar = format!("width:{pct:.1}%");
                    let color = if is_renewable(&s.source_type) { "bg-aurora-green" } else { "bg-aurora-teal" };
                    rsx! {
                        div {
                            div { class: "flex items-baseline justify-between text-sm",
                                span { class: "text-muted", "{s.source_type}" }
                                span { class: "readout text-ink", {format!("{:.0} MW", s.value_mw)} }
                            }
                            div { class: "mt-1.5 h-1.5 w-full overflow-hidden rounded-full bg-elevated",
                                div { class: "h-full rounded-full {color}", style: "{bar}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn FlowTable(data: Vec<FlowPoint>) -> Element {
    rsx! {
        table { class: "mt-5 w-full text-left text-sm",
            thead {
                tr { class: "eyebrow text-faint",
                    th { class: "pb-2 font-semibold", "Border" }
                    th { class: "pb-2 font-semibold", "Direction" }
                    th { class: "pb-2 text-right font-semibold", "MW" }
                }
            }
            tbody {
                for f in data {
                    tr { class: "border-t border-line",
                        td { class: "py-2 text-ink", "{f.from_area} \u{2192} {f.to_area}" }
                        td { class: "py-2",
                            if f.to_area == "FI" {
                                span { class: "rounded-full bg-aurora-green/10 px-2 py-0.5 text-xs text-aurora-green", "Import" }
                            } else {
                                span { class: "rounded-full bg-aurora-teal/10 px-2 py-0.5 text-xs text-aurora-teal", "Export" }
                            }
                        }
                        td { class: "py-2 text-right readout text-muted", {format!("{:.0}", f.value_mw)} }
                    }
                }
            }
        }
    }
}
