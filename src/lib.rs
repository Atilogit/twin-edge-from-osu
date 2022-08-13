mod convert;
mod osu;
mod te;

use std::str::FromStr;

use anyhow::anyhow;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlElement, HtmlInputElement, HtmlSelectElement, Request, RequestInit};

use crate::convert::SpinnerBehaviour;

#[wasm_bindgen]
pub fn run() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Trace).expect("Couldn't initialize logger");
    log::trace!("Initialized WASM")
}

#[wasm_bindgen]
pub async fn convert_url() {
    hide_error();

    // Read inputs
    let doc = web_sys::window()
        .and_then(|w| w.document())
        .expect("Error getting document");
    let url_str = doc
        .get_element_by_id("osu_url")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .expect("Error getting url")
        .value();
    let url = url::Url::from_str(&url_str);

    let slider_beat_limit: f32 = doc
        .get_element_by_id("slider_beat_limit")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .expect("Error getting slider limit")
        .value()
        .parse()
        .unwrap();
    let crop_thumb: bool = doc
        .get_element_by_id("crop_thumb")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .expect("Error getting crop toggle")
        .checked();
    let spinner_behaviour: SpinnerBehaviour = match doc
        .get_element_by_id("spinner_behaviour")
        .and_then(|e| e.dyn_into::<HtmlSelectElement>().ok())
        .expect("Error getting spinner behaviour")
        .value()
        .as_str()
    {
        "ignore" => SpinnerBehaviour::Ignore,
        "current" => SpinnerBehaviour::Current,
        _ => SpinnerBehaviour::Both,
    };

    let mask_normal: bool = doc
        .get_element_by_id("mask_normal")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .expect("Error getting normal mask")
        .checked();
    let mask_whistle: bool = doc
        .get_element_by_id("mask_whistle")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .expect("Error getting whistle mask")
        .checked();
    let mask_finish: bool = doc
        .get_element_by_id("mask_finish")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .expect("Error getting finish mask")
        .checked();
    let mask_clap: bool = doc
        .get_element_by_id("mask_clap")
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .expect("Error getting clap mask")
        .checked();

    let mut reverse_hitsound_mask = 0;
    if mask_normal {
        reverse_hitsound_mask |= 1 << 0;
    }
    if mask_whistle {
        reverse_hitsound_mask |= 1 << 1;
    }
    if mask_finish {
        reverse_hitsound_mask |= 1 << 2;
    }
    if mask_clap {
        reverse_hitsound_mask |= 1 << 3;
    }

    // Start conversion
    if url.is_err() {
        panic_with("Invalid url").await;
    }
    let url = url.unwrap();
    log::trace!("Converting url {url}");
    show_message(&format!("Converting url {url}"));
    log_discord(&format!("Converting url {url}")).await.unwrap();

    if url.domain().is_none()
        || url.domain().unwrap_or("") != "osu.ppy.sh"
        || !url.path().starts_with("/beatmapsets/")
    {
        panic_with("Only urls in the form 'https://osu.ppy.sh/beatmapsets/{set_id}#osu/{map_id}' are supported").await;
    }

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

        show_message("Downloading...");
        let map_file = osu::download(set_id).await;
        if let Ok(map_file) = map_file {
            let diff = map_file
                .into_iter()
                .find(|m| m.data.metadata.beatmap_id == map_id as i32)
                .unwrap();
            show_message("Converting...");
            let te_map = convert::convert(
                &diff,
                slider_beat_limit,
                reverse_hitsound_mask,
                crop_thumb,
                spinner_behaviour,
            );
            show_message("Generating zip...");
            let zip = te_map.as_zip().unwrap();
            show_message("Saving...");
            download_file(
                &format!(
                    "{} {} ({}).zip",
                    te_map.data.artist, te_map.data.display_name, te_map.data.mapper_name
                ),
                &zip,
            );
            show_message("Done");
            hide_error();
            log_discord(&format!("Converted {map_id}")).await.unwrap();
        } else {
            log::trace!("{map_file:?}");
            panic_with("Could not download map file").await;
        }
    } else {
        panic_with("Converting multiple difficulties not supported yet").await;
    }
}

#[wasm_bindgen]
pub fn convert_file() {
    todo!()
}

async fn panic_with(err: &str) {
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

fn show_message(text: &str) {
    log::trace!("{}", text);
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("error-container"))
        .map(|e| {
            e.set_text_content(Some(text));
            e.set_attribute("style", "")
                .unwrap_or_else(|_| panic!("Error displaying text: {text}"));
        })
        .unwrap_or_else(|| panic!("Error displaying text: {text}"));
}

fn download_file(name: &str, content: &[u8]) {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.create_element("a").ok())
        .and_then(|e| {
            e.set_attribute("download", name).ok()?;
            e.set_attribute(
                "href",
                &("data:application/octet-stream;base64,".to_owned() + &base64::encode(content)),
            )
            .ok()?;
            e.dyn_into::<HtmlElement>().ok()?.click();
            Some(())
        })
        .expect("Error saving file")
}

async fn log_discord(text: &str) -> anyhow::Result<()> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    let headers = js_sys::Map::new();
    headers.set(
        &JsValue::from("Content-Type"),
        &JsValue::from("application/json"),
    );
    opts.headers(&headers);
    opts.body(Some(&JsValue::from(format!(r#"{{"content":{text:?}}}"#))));

    let url = "https://canary.discord.com/api/webhooks/1004102414514794575/W6gQnTzto5X-ym-obx2YyYzJ8JLYc8sIfdWTamoFnMTB63loihVRZq64U7ztKRHKI0i2";
    let request = Request::new_with_str_and_init(url, &opts).map_err(|e| anyhow!("{:?}", e))?;

    let window = web_sys::window().unwrap();
    JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| anyhow!("{:?}", e))?;

    Ok(())
}
