mod osu;
mod te;

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::HtmlInputElement;

#[wasm_bindgen]
pub fn run() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Trace).expect("Couldn't initialize logger");
    log::trace!("Initialized WASM")
}

#[wasm_bindgen]
pub fn convert_url() {
    let doc = web_sys::window()
        .and_then(|w| w.document())
        .expect("Error getting document");
    let url = doc
        .get_element_by_id("osu_url")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .expect("Error getting url")
        .value();
    log::trace!("Converting url {url:?}");
}

#[wasm_bindgen]
pub fn convert_file() {
    todo!()
}
