mod osu;
mod te;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
    log::info!("Initialized WASM")
}

#[wasm_bindgen]
pub fn test() {
    log::info!("Click me daddy");
}
