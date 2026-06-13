use dioxus::prelude::*;
use super::app::Route;

#[component]
pub fn Nav() -> Element {
    let links = [
        (Route::Overview {}, "Overview"),
        (Route::Prices {}, "Prices"),
        (Route::Grid {}, "Grid"),
    ];
    let current: Route = use_route();
    rsx! {
        header { class: "sticky top-0 z-20 border-b border-line bg-base/70 backdrop-blur-md",
            div { class: "mx-auto flex max-w-6xl items-center justify-between px-6 py-3.5",
                Link { to: Route::Overview {}, class: "group flex items-center gap-2.5",
                    span { class: "grid h-7 w-7 place-items-center rounded-md border border-aurora-green/30 bg-aurora-green/10 text-aurora-green animate-glow-pulse",
                        "\u{26A1}"
                    }
                    div { class: "leading-none",
                        div { class: "font-display text-[0.95rem] font-extrabold tracking-tight text-ink",
                            "VOLTTI"
                        }
                        div { class: "eyebrow mt-0.5 text-faint", "FINNISH ELECTRICITY" }
                    }
                }
                nav { class: "flex items-center gap-1",
                    for (route , label) in links {
                        Link {
                            to: route.clone(),
                            class: if current == route {
                                "rounded-lg px-3.5 py-1.5 text-sm font-semibold text-aurora-green bg-aurora-green/10 transition"
                            } else {
                                "rounded-lg px-3.5 py-1.5 text-sm font-medium text-muted hover:text-ink hover:bg-elevated transition"
                            },
                            "{label}"
                        }
                    }
                }
                div { class: "hidden items-center gap-2 sm:flex",
                    span { class: "h-1.5 w-1.5 rounded-full bg-aurora-green animate-glow-pulse" }
                    span { class: "eyebrow text-faint", "ENTSO-E LIVE" }
                }
            }
        }
        main { class: "mx-auto max-w-6xl px-6 py-8",
            Outlet::<Route> {}
        }
        footer { class: "border-t border-line py-6 text-center text-sm text-muted",
            "© Jere Niemi, 2026"
        }
    }
}
