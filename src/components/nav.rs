use dioxus::prelude::*;
use super::app::Route;

#[component]
pub fn Nav() -> Element {
    let links = [
        (Route::Overview {}, "Overview"),
        (Route::Prices {}, "Prices"),
        (Route::Generation {}, "Generation"),
        (Route::Forecast {}, "Forecast"),
        (Route::Flows {}, "Flows"),
    ];
    let current: Route = use_route::<Route>();
    rsx! {
        nav { class: "flex items-center gap-6 bg-gray-900 px-6 py-4 text-gray-100 border-b border-gray-800",
            span { class: "text-lg font-bold text-emerald-400", "FI Energy Dashboard" }
            div { class: "flex gap-2",
                for (route, label) in links {
                    Link {
                        to: route.clone(),
                        class: if current == route { "px-3 py-1 rounded bg-gray-700 text-white" } else { "px-3 py-1 rounded text-gray-300 hover:bg-gray-800" },
                        "{label}"
                    }
                }
            }
        }
        main { class: "min-h-screen bg-gray-900 p-6 text-gray-100",
            Outlet::<Route> {}
        }
    }
}
