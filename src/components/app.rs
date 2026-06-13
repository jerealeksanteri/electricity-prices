use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    rsx! {
        div { class: "min-h-screen bg-gray-900 text-gray-100 p-8",
            h1 { class: "text-2xl font-bold", "FI Energy Dashboard" }
        }
    }
}