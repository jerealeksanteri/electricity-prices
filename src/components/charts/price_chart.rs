use dioxus::prelude::*;
use charming::{
    Chart,
    component::Axis,
    element::{AxisType, Tooltip, Trigger},
    series::Bar,
};

use crate::server::PricePoint;
use super::{next_id, render_echarts};

fn eur_mwh_to_c_kwh(p: f64) -> f64 {
    p / 10.0
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

#[component]
pub fn PriceChart(data: Vec<PricePoint>) -> Element {
    let id = use_memo(|| format!("price-chart-{}", next_id()));
    use_effect(use_reactive!(|data| {
        let labels: Vec<String> = data
            .iter()
            .map(|p| p.timestamp.format("%a %H:%M").to_string())
            .collect();
        let values: Vec<f64> = data.iter().map(|p| p.price_eur_mwh).collect();
        let chart = Chart::new()
            .tooltip(Tooltip::new().trigger(Trigger::Axis))
            .x_axis(Axis::new().type_(AxisType::Category).data(labels.clone()))
            .y_axis(Axis::new().type_(AxisType::Value))
            .series(Bar::new().data(values.clone()));
        let mut json = serde_json::to_string(&chart).unwrap_or_else(|_| "{}".into());
        // Recolor each bar by price tier via the serialized series `data` array.
        let colored: Vec<String> = values
            .iter()
            .map(|v| {
                format!(
                    "{{\"value\":{},\"itemStyle\":{{\"color\":\"{}\",\"borderRadius\":[3,3,0,0]}}}}",
                    v,
                    bar_color(eur_mwh_to_c_kwh(*v))
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
        render_echarts(&id(), &json);
    }));
    rsx! { div { id: "{id}", class: "h-72 w-full" } }
}
