//! Voxel / massing **presentation** dump series (counts only, no verdicts).
//!
//! Plan: `plan_voxel_massing_presentation_v1.md` · `plan_ref_books_upgrade_v3.md` D0.
//! Reads [`ConstructionVisualRequests::massing_columns`] RO — never writes visual authority.

use std::collections::BTreeMap;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde_json::{json, Value};

use crate::construction::{ConstructionVisualRequests, StudioMassingOverride};
use crate::gui::WorldPresentationMode;
use crate::render::pixel_world::PixelWorldCameraFrame;

#[derive(Clone, Copy, Debug, Default)]
struct Series {
    first: Option<f64>,
    last: f64,
    min: f64,
    max: f64,
}

impl Series {
    fn push(&mut self, v: f64) {
        if self.first.is_none() {
            self.first = Some(v);
            self.min = v;
            self.max = v;
        }
        self.last = v;
        self.min = self.min.min(v);
        self.max = self.max.max(v);
    }

    fn json(&self) -> Value {
        json!({
            "first": self.first.unwrap_or(0.0),
            "last": self.last,
            "min": self.min,
            "max": self.max,
        })
    }
}

/// Run-over-run massing / presentation counters (sole writer: [`sample_presentation_series`]).
#[derive(Resource, Debug, Default)]
pub struct PresentationDumpSeries {
    pub samples: u64,
    pub kind: String,
    pub pixel_world_active: bool,
    pub pixel_px_per_tile: f32,
    series: BTreeMap<&'static str, Series>,
}

fn track(s: &mut PresentationDumpSeries, key: &'static str, v: f64) {
    s.series.entry(key).or_default().push(v);
}

#[derive(SystemParam)]
pub struct PresentationSampleReads<'w> {
    requests: Option<Res<'w, ConstructionVisualRequests>>,
    studio: Option<Res<'w, StudioMassingOverride>>,
    mode: Option<Res<'w, WorldPresentationMode>>,
    pixel: Option<Res<'w, PixelWorldCameraFrame>>,
}

pub fn sample_presentation_series(mut s: ResMut<PresentationDumpSeries>, r: PresentationSampleReads) {
    if !super::sim_run_dump::sim_dump_enabled(None) {
        return;
    }
    s.samples = s.samples.saturating_add(1);
    s.kind = r
        .mode
        .as_ref()
        .map(|m| format!("{:?}", m.kind))
        .unwrap_or_else(|| "Unknown".into());
    let pixel_world_active = r.pixel.as_ref().is_some_and(|p| p.active);
    s.pixel_world_active = pixel_world_active;
    s.pixel_px_per_tile = r.pixel.as_ref().map(|p| p.px_per_tile).unwrap_or(0.0);

    let (cols, floors_max, floors_sum, footprint_tiles) = r.requests.as_ref().map_or((0, 0, 0, 0), |req| {
        let cols = req.massing_columns.len();
        let floors_max = req.massing_columns.iter().map(|c| u64::from(c.floors)).max().unwrap_or(0);
        let floors_sum: u64 = req.massing_columns.iter().map(|c| u64::from(c.floors)).sum();
        let footprint_tiles = req.footprint_tiles.len();
        (cols, floors_max, floors_sum, footprint_tiles)
    });

    track(&mut s, "massing_columns", cols as f64);
    track(&mut s, "massing_floors_max", floors_max as f64);
    track(&mut s, "massing_floors_sum", floors_sum as f64);
    track(&mut s, "massing_footprint_tiles", footprint_tiles as f64);
    track(
        &mut s,
        "massing_studio_override",
        if r.studio.as_ref().is_some_and(|st| st.is_active()) {
            1.0
        } else {
            0.0
        },
    );
    track(
        &mut s,
        "voxel_pixel_world_active",
        if pixel_world_active { 1.0 } else { 0.0 },
    );
}

/// `presentation` block of the dump (`samples: 0` when the collector never ran).
#[must_use]
pub fn presentation_json(s: Option<&PresentationDumpSeries>) -> Value {
    let Some(s) = s else {
        return json!({ "samples": 0 });
    };
    let samples = s.samples;
    let kind = s.kind.clone();
    let pixel_world_active = s.pixel_world_active;
    let pixel_px_per_tile = s.pixel_px_per_tile;
    let series: Vec<(&'static str, Value)> = s.series.iter().map(|(k, v)| (*k, v.json())).collect();
    let mut out = serde_json::Map::new();
    out.insert("samples".into(), json!(samples));
    out.insert("kind".into(), json!(kind));
    out.insert("pixel_world_active".into(), json!(pixel_world_active));
    out.insert("pixel_px_per_tile".into(), json!(pixel_px_per_tile));
    for (k, v) in series {
        out.insert(k.into(), v);
    }
    Value::Object(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_block_has_no_green_verdict() {
        let mut s = PresentationDumpSeries::default();
        s.samples = 1;
        s.kind = "PixelWorld3D".into();
        track(&mut s, "massing_columns", 2.0);
        let v = presentation_json(Some(&s));
        assert_eq!(v["samples"], 1);
        assert_eq!(v["massing_columns"]["last"], 2.0);
        assert!(v.get("green").is_none());
    }
}

pub struct PresentationDumpPlugin;

impl Plugin for PresentationDumpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PresentationDumpSeries>()
            .add_systems(Last, sample_presentation_series);
    }
}