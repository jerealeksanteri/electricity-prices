mod components;
#[cfg(feature = "server")]
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
    // Real bootstrap is implemented in a later task. For now just launch.
    dioxus::launch(App);
}