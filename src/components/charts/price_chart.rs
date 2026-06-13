use dioxus::prelude::*;
use charming::{
    Chart,
    component::Axis,
    element::{AxisType, Tooltip, Trigger},
    series::Bar,
};

use crate::server::PricePoint;
use super::{next_id, render_echarts};

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

fn build_price_json(data: &[PricePoint]) -> String {
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
    json
}

#[component]
pub fn PriceChart(data: Vec<PricePoint>) -> Element {
    let id = use_hook(|| format!("price-chart-{}", next_id()));
    let mut rendered = use_signal(|| false);

    // Build chart JSON eagerly so both onmounted and effect use the same data.
    let json = build_price_json(&data);
    let mut json_signal = use_signal(|| json.clone());

    // Keep the signal in sync with incoming data.
    if *json_signal.peek() != json {
        json_signal.set(json.clone());
    }

    // Trigger render once the element is mounted (handles first load / hydration).
    let id_clone = id.clone();
    let onmount = move |_: Event<MountedData>| {
        render_echarts(&id_clone, &json_signal());
        rendered.set(true);
    };

    // Re-render when data changes after mount.
    let id_effect = id.clone();
    use_effect(move || {
        let json = json_signal();
        if rendered() {
            render_echarts(&id_effect, &json);
        }
    });

    rsx! { div { id: "{id}", class: "h-72 w-full", onmounted: onmount } }
}
