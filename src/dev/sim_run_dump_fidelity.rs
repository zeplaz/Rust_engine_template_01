//! FIDELITY / MISSILE dump series (counts only) for PLAYCHECK-1 matrix rows.
//! CQ-68: per-domain sample systems write the shared series (no cross-domain SystemParam).

use std::collections::BTreeMap;

use bevy::prelude::*;
use serde_json::{json, Value};

use crate::audio::AudioMixFrame;
use crate::combat::{BallisticFlightBook, BallisticPresentFrame, MissileSortieDump};
use crate::construction::BlastCraterDump;
use crate::presentation::truck_view::TruckPresentationIndex;

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

/// Run-over-run fidelity / sensory counters.
#[derive(Resource, Debug, Default)]
pub struct FidelityDumpSeries {
    pub samples: u64,
    series: BTreeMap<&'static str, Series>,
}

fn track(s: &mut FidelityDumpSeries, key: &'static str, v: f64) {
    s.series.entry(key).or_default().push(v);
}

fn bump_sample(s: &mut FidelityDumpSeries) {
    s.samples = s.samples.saturating_add(1);
}

/// Combat / blast fidelity samples.
pub fn sample_fidelity_combat_series(
    mut s: ResMut<FidelityDumpSeries>,
    missile: Option<Res<MissileSortieDump>>,
    ballistics: Option<Res<BallisticFlightBook>>,
    ballistic_present: Option<Res<BallisticPresentFrame>>,
    blast: Option<Res<BlastCraterDump>>,
) {
    if !super::sim_run_dump::sim_dump_enabled(None) {
        return;
    }
    bump_sample(&mut s);
    if let Some(m) = missile.as_ref() {
        track(&mut s, "missile_dump_present", 1.0);
        track(&mut s, "missile_sorties_ok", m.sorties_ok as f64);
        track(&mut s, "missile_sorties_aborted_wx", m.sorties_aborted_wx as f64);
        track(
            &mut s,
            "missile_sorties_total",
            (m.sorties_ok + m.sorties_aborted_wx) as f64,
        );
    } else {
        track(&mut s, "missile_dump_present", 0.0);
    }
    if let Some(b) = ballistics.as_ref() {
        track(&mut s, "ballistic_flights", b.flights.len() as f64);
        track(&mut s, "ballistic_impacts", b.impacts as f64);
    } else {
        track(&mut s, "ballistic_flights", 0.0);
        track(&mut s, "ballistic_impacts", 0.0);
    }
    if let Some(p) = ballistic_present.as_ref() {
        track(&mut s, "ballistic_present_trails", p.trail_segments.len() as f64);
        track(&mut s, "ballistic_present_impacts", p.impacts.len() as f64);
        track(&mut s, "ballistic_present_in_flight", f64::from(p.in_flight));
    } else {
        track(&mut s, "ballistic_present_trails", 0.0);
        track(&mut s, "ballistic_present_impacts", 0.0);
        track(&mut s, "ballistic_present_in_flight", 0.0);
    }
    if let Some(c) = blast.as_ref() {
        track(&mut s, "blast_crater_tiles", f64::from(c.tile_count));
        track(&mut s, "blast_applied", c.blasts_applied as f64);
    } else {
        track(&mut s, "blast_crater_tiles", 0.0);
        track(&mut s, "blast_applied", 0.0);
    }
}

/// Audio / truck presentation fidelity samples.
pub fn sample_fidelity_audio_trucks_series(
    mut s: ResMut<FidelityDumpSeries>,
    audio: Option<Res<AudioMixFrame>>,
    trucks: Option<Res<TruckPresentationIndex>>,
) {
    if !super::sim_run_dump::sim_dump_enabled(None) {
        return;
    }
    if let Some(a) = audio.as_ref() {
        track(&mut s, "audio_mix_requests", a.requests.len() as f64);
        track(&mut s, "audio_ambient_rain", f64::from(a.ambient_rain));
    } else {
        track(&mut s, "audio_mix_requests", 0.0);
    }
    let truck_n = trucks.as_ref().map_or(0, |t| t.by_vehicle.len());
    track(&mut s, "truck_proxies", truck_n as f64);
}

#[must_use]
pub fn fidelity_json(s: Option<&FidelityDumpSeries>) -> Value {
    let Some(s) = s else {
        return json!({ "samples": 0 });
    };
    let mut out = serde_json::Map::new();
    out.insert("samples".into(), json!(s.samples));
    for (k, v) in &s.series {
        out.insert((*k).into(), v.json());
    }
    Value::Object(out)
}

pub struct FidelityDumpPlugin;

impl Plugin for FidelityDumpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FidelityDumpSeries>().add_systems(
            Last,
            (
                sample_fidelity_combat_series,
                sample_fidelity_audio_trucks_series.after(sample_fidelity_combat_series),
            ),
        );
    }
}
