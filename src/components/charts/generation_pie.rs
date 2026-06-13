use dioxus::prelude::*;
use charming::{
    Chart,
    component::Legend,
    element::{Tooltip, Trigger},
    series::Pie,
};

use crate::server::GenerationMix;
use super::{next_id, render_echarts};

fn build_pie_json(data: &GenerationMix) -> String {
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
    serde_json::to_string(&chart).unwrap_or_else(|_| "{}".into())
}

#[component]
pub fn GenerationPie(data: GenerationMix) -> Element {
    let id = use_hook(|| format!("gen-pie-{}", next_id()));
    let mut rendered = use_signal(|| false);

    let json = build_pie_json(&data);
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

    rsx! { div { id: "{id}", class: "h-[420px] w-full", onmounted: onmount } }
}
