use dioxus::prelude::*;

use crate::server::{entso::get_consumption_forecast, FI_AREA};
use crate::components::common::{Skeleton, ErrorBanner, Card};
use crate::components::charts::forecast_line::ForecastLine;

#[component]
pub fn Forecast() -> Element {
    let data = use_server_future(|| get_consumption_forecast(FI_AREA.to_string()))?;
    rsx! {
        h1 { class: "mb-4 text-2xl font-bold", "Consumption forecast" }
        SuspenseBoundary {
            fallback: |_ctx: SuspenseContext| rsx! { Skeleton {} },
            match data() {
                Some(Ok(d)) => rsx! {
                    Card { title: "Next 24h load forecast (MW)".to_string(), ForecastLine { data: d } }
                },
                Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                None => rsx! { Skeleton {} },
            }
        }
    }
}
