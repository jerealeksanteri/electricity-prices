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