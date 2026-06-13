use dioxus::prelude::*;

use crate::server::{entso::get_cross_border_flows, FI_AREA, FlowPoint};
use crate::components::common::{Skeleton, ErrorBanner, Card};
use crate::components::charts::flow_map::FlowChart;

#[component]
pub fn Flows() -> Element {
    let data = use_server_future(|| get_cross_border_flows(FI_AREA.to_string()))?;
    rsx! {
        h1 { class: "mb-4 text-2xl font-bold", "Cross-border flows" }
        SuspenseBoundary {
            fallback: |_ctx: SuspenseContext| rsx! { Skeleton {} },
            match data() {
                Some(Ok(d)) => rsx! {
                    Card { title: "Net physical flows (MW)".to_string(), FlowChart { data: d.clone() } }
                    FlowTable { data: d }
                },
                Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                None => rsx! { Skeleton {} },
            }
        }
    }
}

#[component]
fn FlowTable(data: Vec<FlowPoint>) -> Element {
    rsx! {
        table { class: "mt-4 w-full text-left text-sm",
            thead {
                tr { class: "text-gray-400",
                    th { class: "py-2", "Direction" }
                    th { class: "py-2", "Flow" }
                    th { class: "py-2", "MW" }
                }
            }
            tbody {
                for f in data {
                    tr { class: "border-t border-gray-800",
                        td { class: "py-2", "{f.from_area} -> {f.to_area}" }
                        td { class: "py-2",
                            if f.to_area == "FI" {
                                span { class: "text-emerald-400", "Import" }
                            } else {
                                span { class: "text-amber-400", "Export" }
                            }
                        }
                        td { class: "py-2", "{f.value_mw:.0}" }
                    }
                }
            }
        }
    }
}
