use std::io::{Cursor, Read};

use anyhow::{anyhow, Result};
use image::DynamicImage;
use js_sys::{ArrayBuffer, DataView};
use osuparse::parse_beatmap;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, Request, RequestInit, Response};

use super::Map;

pub async fn download(set_id: u64) -> Result<Vec<Map>> {
    let mut opts = RequestInit::new();
    opts.method("GET");

    let url = format!("https://chimu.moe/d/{set_id}");
    log::trace!("Downloading {url}");

    let request = Request::new_with_str_and_init(&url, &opts).map_err(|e| anyhow!("{:?}", e))?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| anyhow!("{:?}", e))?;
    let resp: Response = resp_value.dyn_into().map_err(|e| anyhow!("{:?}", e))?;

    let blob: Blob = JsFuture::from(resp.blob().map_err(|e| anyhow!("{:?}", e))?)
        .await
        .map_err(|e| anyhow!("{:?}", e))?
        .dyn_into()
        .map_err(|e| anyhow!("{:?}", e))?;

    let buffer: ArrayBuffer = JsFuture::from(blob.array_buffer())
        .await
        .map_err(|e| anyhow!("{:?}", e))?
        .dyn_into()
        .map_err(|e| anyhow!("{:?}", e))?;

    let view = DataView::new(&buffer, 0, buffer.byte_length() as _);

    let data: Vec<u8> = (0..buffer.byte_length())
        .map(|i| view.get_uint8(i as _))
        .collect();

    log::trace!("Downloaded {} bytes", data.len());

    let mut archive = zip::ZipArchive::new(Cursor::new(&data))?;

    let mut maps = Vec::new();
    let mut thumb = None;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name().ends_with(".osu") {
            log::trace!("Found {}", file.name());

            let mut file_data = Vec::with_capacity(file.size() as _);
            file.read_to_end(&mut file_data)?;
            let osu_data = std::str::from_utf8(&file_data)?;

            let data = parse_beatmap(osu_data).map_err(|e| anyhow!(e))?;

            maps.push(Map {
                data,
                audio: Vec::new(),
                thumb: DynamicImage::new_rgb8(1, 1),
            })
        } else {
            let mut image_data = Vec::with_capacity(file.size() as _);
            file.read_to_end(&mut image_data)?;
            if let Ok(img) = image::load_from_memory(&image_data) {
                thumb = Some(img);
            }
        }
    }

    for map in &mut maps {
        if let Some(thumb) = &thumb {
            map.thumb = thumb.to_owned();
        }
        let mut audio = archive.by_name(&map.data.general.audio_filename)?;
        let mut audio_data = Vec::with_capacity(audio.size() as _);
        audio.read_to_end(&mut audio_data)?;
        map.audio = audio_data;
    }

    log::trace!("Imported {} difficulties", maps.len());

    Ok(maps)
}
