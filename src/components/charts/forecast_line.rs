use dioxus::prelude::*;
use charming::{
    Chart,
    component::Axis,
    element::{AreaStyle, AxisType, Tooltip, Trigger},
    series::Line,
};

use crate::server::ForecastPoint;
use super::{next_id, render_echarts};

#[component]
pub fn ForecastLine(data: Vec<ForecastPoint>) -> Element {
    let id = use_memo(|| format!("forecast-{}", next_id()));
    use_effect(use_reactive!(|data| {
        let labels: Vec<String> = data
            .iter()
            .map(|p| p.timestamp.format("%a %H:%M").to_string())
            .collect();
        let values: Vec<f64> = data.iter().map(|p| p.value_mw).collect();
        let chart = Chart::new()
            .tooltip(Tooltip::new().trigger(Trigger::Axis))
            .x_axis(Axis::new().type_(AxisType::Category).data(labels))
            .y_axis(Axis::new().type_(AxisType::Value))
            .series(
                Line::new()
                    .data(values)
                    .smooth(0.45)
                    .area_style(AreaStyle::new()),
            );
        let json = serde_json::to_string(&chart).unwrap_or_else(|_| "{}".into());
        render_echarts(&id(), &json);
    }));
    rsx! { div { id: "{id}", class: "h-72 w-full" } }
}
