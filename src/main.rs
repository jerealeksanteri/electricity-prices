//! # Voltti
//!
//! A Dioxus 0.6 fullstack dashboard for the Finnish electricity market, showing
//! ENTSO-E day-ahead spot prices (in c/kWh), the generation mix, load forecast,
//! and cross-border flows.
//!
//! - [`server`] — shared data types, the moka cache, and the `#[server]`
//!   functions that fetch and cache ENTSO-E data.
//! - [`components`] — the router, pages, charts, and shared UI.
//!
//! The binary launches the Dioxus client on the web target and an Axum server
//! (with the cache injected as an extension) on the server target.

mod components;
mod server;

use components::app::App;

fn main() {
    #[cfg(feature = "server")]
    server_main();

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[cfg(feature = "server")]
fn server_main() {
    use std::sync::Arc;

    use axum::Extension;
    use dioxus::prelude::*;

    use server::cache::EntsoeCache;

    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let token = std::env::var("ENTSO_E_TOKEN").expect(
        "ENTSO_E_TOKEN must be set — request an API token at https://transparency.entsoe.eu/",
    );
    let cache = Arc::new(EntsoeCache::new(token));

    let rt = tokio::runtime::Runtime::new().expect("create tokio runtime");
    rt.block_on(async move {
        // Pre-fetch and continuously refresh cached data in the background so
        // user requests always hit warm cache entries.
        let ready = server::cache::spawn_cache_warmer(cache.clone());

        // Wait for the first cache warm to complete before accepting traffic.
        // This ensures the very first user request hits a warm cache instead of
        // triggering a duplicate cold-cache fetch.
        tracing::info!("warming caches before accepting traffic\u{2026}");
        ready.notified().await;

        // Under `dx serve` (dev), the Dioxus CLI assigns the server address and
        // proxies to it — honor that. In production, bind BIND_ADDR when set
        // (the Docker image sets it to 0.0.0.0:8080).
        let addr: std::net::SocketAddr = match std::env::var("BIND_ADDR") {
            Ok(s) => s
                .parse()
                .expect("BIND_ADDR must be a valid socket address (e.g. 0.0.0.0:8080)"),
            Err(_) => dioxus::cli_config::fullstack_address_or_localhost(),
        };

        // Build the axum router:
        //  1. serve_dioxus_application registers server functions, static assets, and SSR.
        //  2. .layer(Extension(cache)) wraps every route so that request extensions
        //     contain Arc<EntsoeCache>, which axum::extract::Extension then pulls out
        //     inside #[server] fns via `extract::<Extension<Arc<EntsoeCache>>>()`.
        let router = axum::Router::new()
            .serve_dioxus_application(ServeConfigBuilder::default(), App)
            .layer(Extension(cache));

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("bind to BIND_ADDR");

        tracing::info!("listening on http://{addr}");

        axum::serve(listener, router.into_make_service())
            .await
            .expect("axum server error");
    });
}