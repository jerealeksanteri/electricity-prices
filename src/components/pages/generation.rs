use dioxus::prelude::*;

use crate::server::{entso::get_generation_mix, FI_AREA};
use crate::components::common::{Skeleton, ErrorBanner, Card};
use crate::components::charts::generation_pie::GenerationPie;

#[component]
pub fn Generation() -> Element {
    let data = use_server_future(|| get_generation_mix(FI_AREA.to_string()))?;
    rsx! {
        h1 { class: "mb-4 text-2xl font-bold", "Generation mix" }
        SuspenseBoundary {
            fallback: |_ctx: SuspenseContext| rsx! { Skeleton {} },
            match data() {
                Some(Ok(d)) => rsx! {
                    Card { title: "Current generation by source (MW)".to_string(), GenerationPie { data: d } }
                },
                Some(Err(e)) => rsx! { ErrorBanner { msg: e.to_string() } },
                None => rsx! { Skeleton {} },
            }
        }
    }
}
