use dioxus::prelude::*;
use charming::{
    Chart,
    component::Legend,
    element::{Tooltip, Trigger},
    series::Pie,
};

use crate::server::GenerationMix;
use super::{next_id, render_echarts};

#[component]
pub fn GenerationPie(data: GenerationMix) -> Element {
    let id = use_memo(|| format!("gen-pie-{}", next_id()));
    use_effect(use_reactive!(|data| {
        let pairs: Vec<(f64, String)> = data
            .sources
            .iter()
            .map(|s| (s.value_mw, s.source_type.clone()))
            .collect();
        let chart = Chart::new()
            .tooltip(Tooltip::new().trigger(Trigger::Item))
            .legend(Legend::new().bottom("0").left("center"))
            .series(
                Pie::new()
                    .radius(vec!["46%", "74%"])
                    .center(vec!["50%", "44%"])
                    .data(pairs),
            );
        let json = serde_json::to_string(&chart).unwrap_or_else(|_| "{}".into());
        render_echarts(&id(), &json);
    }));
    rsx! { div { id: "{id}", class: "h-[420px] w-full" } }
}
