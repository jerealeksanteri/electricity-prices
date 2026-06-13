use dioxus::prelude::*;
use charming::{
    Chart,
    component::Axis,
    element::{AxisType, Tooltip, Trigger},
    series::Bar,
};

use crate::server::FlowPoint;
use super::render_echarts;

const CHART_ID: &str = "echart-flows";

#[component]
pub fn FlowChart(data: Vec<FlowPoint>) -> Element {
    use_effect(use_reactive!(|data| {
        let labels: Vec<String> = data
            .iter()
            .map(|f| format!("{} \u{2192} {}", f.from_area, f.to_area))
            .collect();
        let chart = Chart::new()
            .tooltip(Tooltip::new().trigger(Trigger::Axis))
            .x_axis(Axis::new().type_(AxisType::Value))
            .y_axis(Axis::new().type_(AxisType::Category).data(labels))
            .series(Bar::new().data(data.iter().map(|f| f.value_mw).collect::<Vec<f64>>()));
        let mut json = serde_json::to_string(&chart).unwrap_or_else(|_| "{}".into());
        let colored: Vec<String> = data
            .iter()
            .map(|f| {
                let color = if f.to_area == "FI" { "#5ef2a6" } else { "#34d3e0" };
                format!(
                    "{{\"value\":{},\"itemStyle\":{{\"color\":\"{}\",\"borderRadius\":[0,3,3,0]}}}}",
                    f.value_mw, color
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
    rsx! { div { id: CHART_ID, class: "h-64 w-full" } }
}
