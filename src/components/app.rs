use dioxus::prelude::*;

use super::nav::Nav;
use super::pages::{grid::Grid, overview::Overview, prices::Prices};

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(Nav)]
    #[route("/")]
    Overview {},
    #[route("/prices")]
    Prices {},
    #[route("/grid")]
    Grid {},
}

const TAILWIND: Asset = asset!("/assets/tailwind.css");
const ECHARTS: Asset = asset!("/assets/echarts.min.js");

#[component]
pub fn App() -> Element {
    rsx! {
        document::Stylesheet { href: TAILWIND.to_string() }
        document::Script { src: ECHARTS.to_string() }
        // Nordic-technical type: Schibsted Grotesk display + IBM Plex Mono data.
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link { rel: "preconnect", href: "https://fonts.gstatic.com", crossorigin: "anonymous" }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600&family=Schibsted+Grotesk:wght@400;500;600;700;800&display=swap",
        }
        Router::<Route> {}
    }
}
