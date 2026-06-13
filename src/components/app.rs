use dioxus::prelude::*;

use super::nav::Nav;
use super::pages::{
    overview::Overview,
    prices::Prices,
    generation::Generation,
    forecast::Forecast,
    flows::Flows,
};

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(Nav)]
    #[route("/")]
    Overview {},
    #[route("/prices")]
    Prices {},
    #[route("/generation")]
    Generation {},
    #[route("/forecast")]
    Forecast {},
    #[route("/flows")]
    Flows {},
}

const TAILWIND: Asset = asset!("/assets/tailwind.css");
const ECHARTS: Asset = asset!("/assets/echarts.min.js");

#[component]
pub fn App() -> Element {
    rsx! {
        document::Stylesheet { href: TAILWIND.to_string() }
        document::Script { src: ECHARTS.to_string() }
        Router::<Route> {}
    }
}
