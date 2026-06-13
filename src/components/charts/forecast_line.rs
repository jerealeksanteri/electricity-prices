use dioxus::prelude::*;
use charming::{Chart, component::Axis, element::AxisType, series::Line};

use crate::server::ForecastPoint;
use super::{next_id, render_echarts};

#[component]
pub fn ForecastLine(data: Vec<ForecastPoint>) -> Element {
    let id = use_memo(|| format!("forecast-{}", next_id()));
    use_effect(move || {
        let labels: Vec<String> = data
            .iter()
            .map(|p| p.timestamp.format("%H:%M").to_string())
            .collect();
        let values: Vec<f64> = data.iter().map(|p| p.value_mw).collect();
        let chart = Chart::new()
            .x_axis(Axis::new().type_(AxisType::Category).data(labels))
            .y_axis(Axis::new().type_(AxisType::Value))
            // smooth takes F: Into<f64>; 0.5 gives a moderate smooth curve
            .series(Line::new().data(values).smooth(0.5_f64));
        let json = serde_json::to_string(&chart).unwrap_or_else(|_| "{}".into());
        render_echarts(&id(), &json);
    });
    rsx! { div { id: "{id}", class: "h-80 w-full" } }
}
