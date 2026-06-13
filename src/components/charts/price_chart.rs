use dioxus::prelude::*;
use charming::{
    Chart,
    component::Axis,
    element::{AxisType, Tooltip, Trigger},
    series::Bar,
};

use crate::server::PricePoint;
use super::render_echarts;

/// €/MWh → c/kWh (1 €/MWh = 0.1 c/kWh).
fn to_c_kwh(eur_mwh: f64) -> f64 {
    eur_mwh / 10.0
}

fn bar_color(c_kwh: f64) -> &'static str {
    if c_kwh < 5.0 {
        "#43e08a"
    } else if c_kwh <= 15.0 {
        "#f5c451"
    } else {
        "#fb7185"
    }
}

/// A fixed DOM id for this chart. Since the router renders one page at a time,
/// at most one `PriceChart` exists in the DOM, so a constant id is safe and
/// avoids SSR / hydration counter-mismatch issues.
const CHART_ID: &str = "echart-price";

#[component]
pub fn PriceChart(data: Vec<PricePoint>) -> Element {
    use_effect(use_reactive!(|data| {
        let labels: Vec<String> = data
            .iter()
            .map(|p| p.timestamp.format("%a %H:%M").to_string())
            .collect();
        let values: Vec<f64> = data.iter().map(|p| to_c_kwh(p.price_eur_mwh)).collect();
        let chart = Chart::new()
            .tooltip(Tooltip::new().trigger(Trigger::Axis))
            .x_axis(Axis::new().type_(AxisType::Category).data(labels.clone()))
            .y_axis(Axis::new().type_(AxisType::Value))
            .series(Bar::new().name("c/kWh").data(values.clone()));
        let mut json = serde_json::to_string(&chart).unwrap_or_else(|_| "{}".into());
        let colored: Vec<String> = values
            .iter()
            .map(|v| {
                format!(
                    "{{\"value\":{:.2},\"itemStyle\":{{\"color\":\"{}\",\"borderRadius\":[3,3,0,0]}}}}",
                    v,
                    bar_color(*v)
                )
            })
            .collect();
        let colored_arr = format!("[{}]", colored.join(","));
        if let Some(pos) = json.rfind("\"data\":[") {
            let start = pos + "\"data\":".len();
            if let Some(end_rel) = json[start..].find(']') {
                let end = start + end_rel + 1;
                json.replace_range(start..end, &colored_arr);
            }
        }
        render_echarts(CHART_ID, &json);
    }));
    rsx! { div { id: CHART_ID, class: "h-72 w-full" } }
}
