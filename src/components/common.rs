use dioxus::prelude::*;

/// Small uppercase tracked label ("instrument panel" eyebrow).
#[component]
pub fn Eyebrow(text: String) -> Element {
    rsx! { div { class: "eyebrow", "{text}" } }
}

/// Titled panel container.
#[component]
pub fn Card(title: String, children: Element) -> Element {
    rsx! {
        section { class: "panel animate-fade-up p-5",
            div { class: "mb-4 flex items-center justify-between",
                Eyebrow { text: title }
            }
            {children}
        }
    }
}

/// A single statistic tile: tracked label + large mono readout + optional hint.
#[component]
pub fn StatTile(
    label: String,
    value: String,
    #[props(default)] hint: String,
    #[props(default)] accent: String,
) -> Element {
    let color = if accent.is_empty() { "text-ink" } else { accent.as_str() };
    let value_class = format!("readout mt-2 text-3xl font-semibold {color}");
    rsx! {
        div { class: "panel animate-fade-up p-5",
            Eyebrow { text: label }
            div { class: "{value_class}", "{value}" }
            if !hint.is_empty() {
                div { class: "mt-1 text-xs text-faint", "{hint}" }
            }
        }
    }
}

/// Loading placeholder with an aurora shimmer sweep.
#[component]
pub fn Skeleton() -> Element {
    rsx! {
        div { class: "panel relative overflow-hidden p-5",
            div { class: "h-4 w-1/4 rounded bg-elevated" }
            div { class: "mt-4 h-56 w-full rounded-lg bg-elevated/60" }
            div {
                class: "pointer-events-none absolute inset-0 -translate-x-full",
                style: "background: linear-gradient(90deg, transparent, rgba(94,242,166,0.06), transparent); animation: shimmer 1.6s infinite;",
            }
        }
    }
}

/// Inline error surface.
#[component]
pub fn ErrorBanner(msg: String) -> Element {
    rsx! {
        div { class: "panel border-tier-high/40 bg-tier-high/5 p-4 text-sm text-tier-high",
            span { class: "font-semibold", "Error \u{2014} " }
            "{msg}"
        }
    }
}

/// Top-of-page banner shown while server data is being fetched / refreshed.
#[component]
pub fn RefreshingBanner() -> Element {
    rsx! {
        div { class: "mb-4 flex items-center gap-3 rounded-xl bg-aurora-green/5 px-4 py-3 text-sm text-aurora-green animate-fade-up",
            // Spinning circle
            svg {
                class: "h-4 w-4 animate-spin",
                xmlns: "http://www.w3.org/2000/svg",
                fill: "none",
                view_box: "0 0 24 24",
                circle {
                    class: "opacity-25",
                    cx: "12", cy: "12", r: "10",
                    stroke: "currentColor",
                    stroke_width: "4",
                }
                path {
                    class: "opacity-75",
                    fill: "currentColor",
                    d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z",
                }
            }
            span { "Refreshing data from ENTSO-E\u{2026}" }
        }
    }
}

