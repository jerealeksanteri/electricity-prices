use dioxus::prelude::*;
use charming::{
    Chart,
    component::Axis,
    element::{AreaStyle, AxisType, Tooltip, Trigger},
    series::Line,
};

use crate::server::ForecastPoint;
use super::{next_id, render_echarts};

fn build_forecast_json(data: &[ForecastPoint]) -> String {
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
    serde_json::to_string(&chart).unwrap_or_else(|_| "{}".into())
}

#[component]
pub fn ForecastLine(data: Vec<ForecastPoint>) -> Element {
    let id = use_hook(|| format!("forecast-{}", next_id()));
    let mut rendered = use_signal(|| false);

    let json = build_forecast_json(&data);
    let mut json_signal = use_signal(|| json.clone());

    if *json_signal.peek() != json {
        json_signal.set(json.clone());
    }

    let id_clone = id.clone();
    let onmount = move |_: Event<MountedData>| {
        render_echarts(&id_clone, &json_signal());
        rendered.set(true);
    };

    let id_effect = id.clone();
    use_effect(move || {
        let json = json_signal();
        if rendered() {
            render_echarts(&id_effect, &json);
        }
    });

    rsx! { div { id: "{id}", class: "h-72 w-full", onmounted: onmount } }
}
