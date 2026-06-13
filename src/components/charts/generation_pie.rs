use dioxus::prelude::*;
use charming::{Chart, series::Pie};

use crate::server::GenerationMix;
use super::{next_id, render_echarts};

#[component]
pub fn GenerationPie(data: GenerationMix) -> Element {
    let id = use_memo(|| format!("gen-pie-{}", next_id()));
    use_effect(move || {
        // charming Pie::data takes Vec<D: Into<DataPoint>>
        // (V: Into<CompositeValue>, S: Into<String>) -> DataPointItem {value, name}
        let pairs: Vec<(f64, String)> = data
            .sources
            .iter()
            .map(|s| (s.value_mw, s.source_type.clone()))
            .collect();
        let chart = Chart::new()
            .series(Pie::new().radius(vec!["40%", "70%"]).data(pairs));
        let json = serde_json::to_string(&chart).unwrap_or_else(|_| "{}".into());
        render_echarts(&id(), &json);
    });
    rsx! { div { id: "{id}", class: "h-80 w-full" } }
}
