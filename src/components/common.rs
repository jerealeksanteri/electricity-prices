use dioxus::prelude::*;

#[component]
pub fn Skeleton() -> Element {
    rsx! {
        div { class: "animate-pulse space-y-4",
            div { class: "h-6 w-1/3 rounded bg-gray-700" }
            div { class: "h-64 w-full rounded bg-gray-800" }
        }
    }
}

#[component]
pub fn ErrorBanner(msg: String) -> Element {
    rsx! {
        div { class: "rounded border border-red-700 bg-red-900/40 p-4 text-red-200",
            span { class: "font-semibold", "Error: " }
            "{msg}"
        }
    }
}

#[component]
pub fn Card(title: String, children: Element) -> Element {
    rsx! {
        div { class: "rounded-lg border border-gray-700 bg-gray-800 p-5",
            h2 { class: "mb-3 text-sm font-medium uppercase tracking-wide text-gray-400", "{title}" }
            {children}
        }
    }
}