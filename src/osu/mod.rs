mod download;

pub use download::*;

use std::{collections::BTreeMap, fmt::Debug, fs, path::Path};

use anyhow::{anyhow, Result};
use image::DynamicImage;
use osuparse::{parse_beatmap, Beatmap};

pub struct Map {
    pub data: Beatmap,
    pub audio: Vec<u8>,
    pub thumb: DynamicImage,
}

impl Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Map")
            .field("audio", &self.audio)
            .field("thumb", &self.thumb)
            .finish()
    }
}

impl Map {
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Map> {
        let data = parse_beatmap(&fs::read_to_string(&path)?).map_err(|e| anyhow!(e))?;

        let audio = fs::read(
            path.as_ref()
                .parent()
                .unwrap()
                .join(&data.general.audio_filename),
        )?;

        let thumb = path
            .as_ref()
            .parent()
            .unwrap()
            .read_dir()
            .unwrap()
            .find_map(|f| image::open(f.unwrap().path()).ok())
            .ok_or_else(|| anyhow!("Could not find thumb file"))?;

        Ok(Map { data, audio, thumb })
    }

    pub fn find_most_used_bpm(&self) -> f64 {
        let timing_points = &self.data.timing_points;

        let mut bpms = BTreeMap::new();

        let mut last_bpm =
            ((1000. / timing_points.first().unwrap().ms_per_beat as f64) * 60. * 1000.).round()
                as i64;
        let mut last_time = timing_points.first().unwrap().offset as u64;
        for p in timing_points {
            if p.ms_per_beat < 0. {
                continue;
            }

            let bpm_key = ((1000. / p.ms_per_beat as f64) * 60. * 1000.).round() as i64;

            bpms.entry(bpm_key).or_insert(0);
            bpms.insert(
                last_bpm,
                bpms.get(&last_bpm).unwrap() + p.offset as u64 - last_time,
            );
            last_bpm = bpm_key;
            last_time = p.offset as u64;
        }

        let max = bpms.iter().max_by_key(|(_, v)| *v).unwrap();
        *max.0 as f64 / 1000.
    }

    pub fn find_kiai(&self) -> Vec<(f32, f32)> {
        let timing_points = &self.data.timing_points;
        let mut kiai = Vec::new();

        let mut kiai_on = false;
        let mut kiai_start = 0.;
        for p in timing_points {
            if p.kiai_mode && !kiai_on {
                kiai_on = true;
                kiai_start = p.offset;
            } else if !p.kiai_mode && kiai_on {
                kiai_on = false;
                kiai.push((kiai_start, p.offset));
            }
        }
        if kiai_on {
            kiai.push((kiai_start, timing_points.last().unwrap().offset));
        }

        kiai
    }
}
