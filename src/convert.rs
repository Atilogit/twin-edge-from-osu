use osuparse::HitObject;
use te::TimingPoint;

use crate::{osu, te};

pub fn convert(osu_map: osu::Map) -> te::Map {
    let bpm = osu_map.find_most_used_bpm();

    let slider_limit = 0.5;
    let reverse_hitsound_mask = 0b1110;

    let thumb = osu_map.thumb.clone().crop(
        (osu_map.thumb.width() - osu_map.thumb.height()) / 2,
        0,
        osu_map.thumb.height(),
        osu_map.thumb.height(),
    );
    let kiai = osu_map
        .find_kiai()
        .iter()
        .map(|(s, e)| te::SpecialSection {
            start_time: *s as _,
            end_time: *e as _,
            start_time_in_sec: *s as f64 / 1000.,
            end_time_in_sec: *e as f64 / 1000.,
        })
        .collect();
    let timing_points = osu_map
        .data
        .timing_points
        .iter()
        .skip(1) // skip first because timing point cannot be on start
        .filter(|p| p.ms_per_beat > 0.)
        .map(|p| TimingPoint {
            time: p.offset as _,
            bpm: (1000. / p.ms_per_beat as f64) * 60.,
            time_in_sec: p.offset as f64 / 1000.,
        })
        .collect();

    let mut left_notes = String::new();
    let mut right_notes = String::new();
    let mut left = true;
    for o in osu_map.data.hit_objects {
        if match &o {
            HitObject::HitCircle(o) => o.new_combo,
            HitObject::Slider(o) => o.new_combo,
            HitObject::Spinner(o) => o.new_combo,
            HitObject::HoldNote(o) => o.new_combo,
        } {
            left = !left;
        }

        let current_side = if left {
            &mut left_notes
        } else {
            &mut right_notes
        };

        match o {
            // a:b:c
            // a time
            // b type: 0: Normal, 1: Reverse, 2: Hold Start, 3: Hold End
            // c angle?
            HitObject::HitCircle(o) => {
                if o.hitsound & reverse_hitsound_mask == reverse_hitsound_mask {
                    current_side.push_str(&format!("{}:1:0|", o.time));
                } else {
                    current_side.push_str(&format!("{}:0:0|", o.time));
                }
            }
            HitObject::Slider(o) => {
                // Slider velocity
                let slider_velocity = 1.
                    / osu_map
                        .data
                        .timing_points
                        .iter()
                        .filter(|p| p.offset <= o.time as f32 + 0.0001 && p.ms_per_beat < 0.)
                        .map(|p| -p.ms_per_beat / 100.)
                        .last()
                        .unwrap_or(1.);
                // BPM
                let beat_length = osu_map
                    .data
                    .timing_points
                    .iter()
                    .filter(|p| p.offset <= o.time as f32 + 0.0001 && p.ms_per_beat > 0.)
                    .map(|p| p.ms_per_beat)
                    .last()
                    .unwrap_or(1.);

                let slider_frac = o.pixel_length
                    / (osu_map.data.difficulty.slider_multiplier * 100. * slider_velocity);

                // If slider is too short replace with normal note
                if slider_frac > slider_limit + 0.001 {
                    let slider_time = (slider_frac * beat_length) as i32;
                    current_side.push_str(&format!(
                        "{}:2:0|{}:3:0|",
                        o.time,
                        o.time + slider_time * o.repeat
                    ));
                } else {
                    current_side.push_str(&format!("{}:0:0|", o.time));
                }
            }
            HitObject::Spinner(o) => {
                left_notes.push_str(&format!("{}:2:0|{}:3:0|", o.time, o.end_time));
                right_notes.push_str(&format!("{}:2:0|{}:3:0|", o.time, o.end_time));
            }
            HitObject::HoldNote(o) => {
                current_side.push_str(&format!("{}:2:0|{}:3:0|", o.time, o.end_time));
            }
        }
    }
    left_notes.pop();
    right_notes.pop();

    let te_map = te::Map {
        data: te::MapData {
            mapper_name: osu_map.data.metadata.creator,
            audio_file_name: osu_map.data.general.audio_filename,
            thumbnail_file_name: "thumb.png".to_string(),
            song_file_name: format!("{}.song", osu_map.data.metadata.title),
            display_name: osu_map.data.metadata.title,
            artist: osu_map.data.metadata.artist,
            difficulty_name: osu_map.data.metadata.version,
            preview_time_seconds: osu_map.data.general.preview_time as f64 / 1000.,
            bpm,
            difficulty_settings: te::DifficultySettings {
                note_appear_time: 0.5,
                rotation_speed: 125.,
                health_drain_per_second: 5.,
            },
            first_beat_offset_in_ms: osu_map.data.timing_points.first().unwrap().offset as _,
            timing_points,
            right_disc_notes: right_notes,
            left_disc_notes: left_notes,
            song_events: Vec::new(),
            special_sections: kiai,
            breaks: Vec::new(),
            additional_difficulties: Vec::new(),
        },
        audio: osu_map.audio,
        thumb,
    };
    te_map
}
