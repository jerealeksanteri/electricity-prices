use dioxus::prelude::*;

use super::nav::Nav;
use super::pages::{grid::Grid, overview::Overview, prices::Prices};

#[derive(Routable, Clone, Debug, PartialEq)]
pub enum Route {
    #[layout(Nav)]
    #[route("/")]
    Overview {},
    #[route("/prices")]
    Prices {},
    #[route("/grid")]
    Grid {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

#[component]
fn NotFound(segments: Vec<String>) -> Element {
    let path = segments.join("/");
    rsx! {
        div { class: "flex flex-col items-center justify-center py-20 text-center",
            h1 { class: "font-display text-5xl font-bold text-ink", "404" }
            p { class: "mt-4 text-lg text-muted", "Page not found: /{path}" }
            Link { to: Route::Overview {}, class: "mt-6 rounded-lg bg-aurora-green/10 px-4 py-2 text-sm font-semibold text-aurora-green hover:bg-aurora-green/20 transition",
                "Back to overview"
            }
        }
    }
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
