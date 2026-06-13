use dioxus::prelude::*;
use charming::{Chart, component::Axis, element::AxisType, series::Bar};

use crate::server::FlowPoint;
use super::{next_id, render_echarts};

#[component]
pub fn FlowChart(data: Vec<FlowPoint>) -> Element {
    let id = use_memo(|| format!("flows-{}", next_id()));
    let rows = data.clone();
    use_effect(move || {
        let labels: Vec<String> = rows
            .iter()
            .map(|f| format!("{}->{}", f.from_area, f.to_area))
            .collect();
        let values: Vec<f64> = rows.iter().map(|f| f.value_mw).collect();
        let chart = Chart::new()
            .x_axis(Axis::new().type_(AxisType::Value))
            .y_axis(Axis::new().type_(AxisType::Category).data(labels))
            .series(Bar::new().data(values));
        let json = serde_json::to_string(&chart).unwrap_or_else(|_| "{}".into());
        render_echarts(&id(), &json);
    });
    rsx! { div { id: "{id}", class: "h-80 w-full" } }
}
