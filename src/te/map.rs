use std::{env, fs, path::Path};

use anyhow::{anyhow, Result};

use image::{DynamicImage, ImageFormat};
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Map {
    pub data: MapData,
    pub audio: Vec<u8>,
    pub thumb: DynamicImage,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapData {
    pub mapper_name: String,
    pub audio_file_name: String,
    pub thumbnail_file_name: String,
    pub song_file_name: String,
    #[serde(rename = "DisplayName")]
    pub display_name: String,
    #[serde(rename = "Artist")]
    pub artist: String,
    #[serde(rename = "DifficultyName")]
    pub difficulty_name: String,
    #[serde(rename = "PreviewTimeSeconds")]
    pub preview_time_seconds: f64,
    #[serde(rename = "Bpm")]
    pub bpm: f64,
    #[serde(rename = "DifficultySettings")]
    pub difficulty_settings: DifficultySettings,
    #[serde(rename = "FirstBeatOffsetInMs")]
    pub first_beat_offset_in_ms: i64,
    #[serde(rename = "TimingPoints")]
    pub timing_points: Vec<TimingPoint>,
    #[serde(rename = "RightDiscNotes")]
    pub right_disc_notes: String,
    #[serde(rename = "LeftDiscNotes")]
    pub left_disc_notes: String,
    #[serde(rename = "SongEvents")]
    pub song_events: Vec<SongEvent>,
    #[serde(rename = "SpecialSections")]
    pub special_sections: Vec<SpecialSection>,
    #[serde(rename = "Breaks")]
    pub breaks: Vec<Break>,
    #[serde(rename = "AdditionalDifficulties")]
    pub additional_difficulties: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DifficultySettings {
    pub note_appear_time: f64,
    pub rotation_speed: f64,
    pub health_drain_per_second: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimingPoint {
    pub time: f64,
    pub bpm: f64,
    pub time_in_sec: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongEvent {
    pub time: f64,
    pub color_change: bool,
    pub color_change_event: ColorChangeEvent,
    pub particles: bool,
    pub particles_event: ParticlesEvent,
    pub special_section: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorChangeEvent {
    pub color: String,
    pub transition_time: f64,
    pub one_shot: bool,
    pub hold_duration: f64,
    pub transition_back_time: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticlesEvent {
    pub duration: f64,
    pub particles: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecialSection {
    pub start_time: f64,
    pub end_time: f64,
    pub start_time_in_sec: f64,
    pub end_time_in_sec: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Break {
    pub start_time: f64,
    pub end_time: f64,
    pub start_time_in_sec: f64,
    pub end_time_in_sec: f64,
}

impl Map {
    #[allow(dead_code)]
    pub fn read(map_search: &str) -> Result<Map> {
        let base_path = Path::new(&env::var_os("userprofile").unwrap())
            .join("AppData")
            .join("LocalLow")
            .join("Arcy")
            .join("TwinEdge")
            .join("CustomSongs");

        let map_dir = base_path
            .read_dir()?
            .find(|f| {
                f.as_ref()
                    .unwrap()
                    .file_name()
                    .into_string()
                    .unwrap()
                    .contains(map_search)
            })
            .ok_or_else(|| anyhow!("Could not find map"))??;

        let data_file = map_dir
            .path()
            .read_dir()?
            .find(|f| {
                f.as_ref()
                    .unwrap()
                    .file_name()
                    .into_string()
                    .unwrap()
                    .ends_with(".song")
            })
            .ok_or_else(|| anyhow!("Could not find .song file"))??;

        let audio_file = map_dir
            .path()
            .read_dir()?
            .find(|f| {
                f.as_ref()
                    .unwrap()
                    .file_name()
                    .into_string()
                    .unwrap()
                    .ends_with(".wav")
            })
            .ok_or_else(|| anyhow!("Could not find audio file"))??;

        let thumb_file = map_dir
            .path()
            .read_dir()?
            .find(|f| {
                f.as_ref()
                    .unwrap()
                    .file_name()
                    .into_string()
                    .unwrap()
                    .ends_with(".png")
            })
            .ok_or_else(|| anyhow!("Could not find thumb file"))??;

        let data = serde_json::from_str(&fs::read_to_string(data_file.path())?)?;

        Ok(Map {
            audio: fs::read(audio_file.path())?,
            data,
            thumb: image::load_from_memory(&fs::read(thumb_file.path())?)?,
        })
    }

    pub fn save(&self) -> Result<()> {
        let base_path = Path::new(&env::var_os("userprofile").unwrap())
            .join("AppData")
            .join("LocalLow")
            .join("Arcy")
            .join("TwinEdge")
            .join("CustomSongs");

        let map_dir = base_path.join(format!(
            "{} {} ({})",
            self.data.artist, self.data.display_name, self.data.mapper_name
        ));

        fs::create_dir_all(&map_dir)?;
        fs::write(
            map_dir.join(&self.data.song_file_name),
            serde_json::to_string_pretty(&self.data)?,
        )?;
        fs::write(map_dir.join(&self.data.audio_file_name), &self.audio)?;
        self.thumb.save_with_format(
            map_dir.join(&self.data.thumbnail_file_name),
            ImageFormat::Png,
        )?;

        Ok(())
    }
}
