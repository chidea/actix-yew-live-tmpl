#![recursion_limit = "512"]

mod app;
mod utils;

use wasm_bindgen::prelude::*;
use yew::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is the entry point for the web app
#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    utils::set_panic_hook();
    web_logger::init();
    yew::initialize();
    let app = App::<app::App>::new();
    let mut scope = app.mount_to_body();
    scope.send_message(app::Msg::Init);
    yew::run_loop();
    // yew::start_app::<app::App>();
    Ok(())
}
