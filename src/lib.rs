mod osu;
mod te;

use std::str::FromStr;

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::HtmlInputElement;

#[wasm_bindgen]
pub fn run() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Trace).expect("Couldn't initialize logger");
    log::trace!("Initialized WASM")
}

#[wasm_bindgen]
pub async fn convert_url() {
    hide_error();

    let doc = web_sys::window()
        .and_then(|w| w.document())
        .expect("Error getting document");
    let url_str = doc
        .get_element_by_id("osu_url")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .expect("Error getting url")
        .value();
    let url = url::Url::from_str(&url_str);

    if let Ok(url) = url {
        log::trace!("Converting url {url:?}");
        if let Some(domain) = url.domain() {
            if domain == "osu.ppy.sh" && url.path().starts_with("/beatmapsets/") {
                let set_id: u64 = url
                    .path_segments()
                    .unwrap()
                    .last()
                    .unwrap()
                    .parse()
                    .unwrap();
                log::trace!("Beatmap set {set_id}");

                let map_id: Option<u64> = url
                    .fragment()
                    .and_then(|f| f.split('/').last())
                    .and_then(|s| s.parse().ok());
                if let Some(map_id) = map_id {
                    log::trace!("Difficulty {map_id}");

                    let map_file = osu::download(set_id).await;
                    if let Ok(map_file) = map_file {
                    } else {
                        log::trace!("{map_file:?}");
                        panic_with("Could not download map file")
                    }
                } else {
                    panic_with("Converting multiple difficulties not supported yet")
                }
            } else {
                panic_with(
					"Only urls in the form 'https://osu.ppy.sh/beatmapsets/{set_id}#osu/{map_id}' are supported",
				)
            }
        } else {
            panic_with(
                "Only urls in the form 'https://osu.ppy.sh/beatmapsets/{set_id}#osu/{map_id}' are supported",
            )
        }
    } else {
        panic_with("Invalid url");
    }
}

#[wasm_bindgen]
pub fn convert_file() {
    todo!()
}

fn panic_with(err: &str) {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("error-container"))
        .map(|e| {
            e.set_text_content(Some(err));
            e.set_attribute("style", "")
                .unwrap_or_else(|_| panic!("Error displaying error: {err}"));
        })
        .unwrap_or_else(|| panic!("Error displaying error: {err}"));
    panic!("{}", err);
}

fn hide_error() {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("error-container"))
        .map(|e| {
            e.set_text_content(None);
            e.set_attribute("style", "display: none;")
                .unwrap_or_else(|_| panic!("Error clearing error"));
        })
        .unwrap_or_else(|| panic!("Error clearing error"));
}
