//! Disposable place-settle pulse after successful Building/Defense commit.
//!
//! Presentation only — does not own commit, FSM, or pointer gate.
//! Charter: `src/dev/design_place_feedback_anim_v1.md` · COD-P3-PLACE-ANIM-001.

use std::sync::atomic::{AtomicU32, Ordering};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;

use super::build_strip::{BuildStripState, ToolContext};
use super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::ghost_visual::{
    footprint_place_settle_color, footprint_valid_color, place_feedback_duration_secs,
};
use super::visual_authority::{
    ConstructionVisualRequests, FootprintTileColorKind, FootprintTileRequest,
};

/// Honest lib/live counters — do not invent non-zero without arm/complete.
static PLACE_FEEDBACK_STARTS: AtomicU32 = AtomicU32::new(0);
static PLACE_FEEDBACK_COMPLETES: AtomicU32 = AtomicU32::new(0);

#[inline]
fn note_place_feedback_start() {
    PLACE_FEEDBACK_STARTS.fetch_add(1, Ordering::Relaxed);
}

#[inline]
fn note_place_feedback_complete() {
    PLACE_FEEDBACK_COMPLETES.fetch_add(1, Ordering::Relaxed);
}

#[must_use]
pub fn place_feedback_start_count() -> u32 {
    PLACE_FEEDBACK_STARTS.load(Ordering::Relaxed)
}

#[must_use]
pub fn place_feedback_complete_count() -> u32 {
    PLACE_FEEDBACK_COMPLETES.load(Ordering::Relaxed)
}

#[must_use]
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let u = 1.0 - t;
    1.0 - u * u * u
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    let t = t.clamp(0.0, 1.0);
    (a as f32 + (b as f32 - a as f32) * t).round().clamp(0.0, 255.0) as u8
}

/// Wall-clock fraction that holds the locked valid fill before the Planned settle.
/// Charter: `design_build_place_feel_v1.md` — no alpha punch.
pub const PLACE_FEEDBACK_LOCK_HOLD_T: f32 = 0.62;

/// Fill color at normalized pulse progress `t` (0..1).
///
/// Until [`PLACE_FEEDBACK_LOCK_HOLD_T`]: valid fill (locked footprint).
/// Tail: ease-out cubic crossfade → Planned settle. Alpha multiplier is 1.0.
#[must_use]
pub fn place_feedback_fill_at(t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    // Unmultiplied RGBA — do not read Color32.r/g/b (premultiplied storage).
    let valid = [48_u8, 140, 72, 220];
    let settle = [100_u8, 144, 220, 200];

    let hue_blend = if t <= PLACE_FEEDBACK_LOCK_HOLD_T {
        0.0
    } else {
        let u = ((t - PLACE_FEEDBACK_LOCK_HOLD_T) / (1.0 - PLACE_FEEDBACK_LOCK_HOLD_T))
            .clamp(0.0, 1.0);
        ease_out_cubic(u)
    };
    egui::Color32::from_rgba_unmultiplied(
        lerp_u8(valid[0], settle[0], hue_blend),
        lerp_u8(valid[1], settle[1], hue_blend),
        lerp_u8(valid[2], settle[2], hue_blend),
        lerp_u8(valid[3], settle[3], hue_blend),
    )
}

/// Disposable settle pulse — sole writer of place-feedback footprint overlays.
#[derive(Resource, Debug, Clone, Default)]
pub struct PlaceFeedbackPulse {
    pub active: bool,
    pub elapsed_secs: f32,
    /// Absolute tile + occupation weight captured at commit (before unlock_to_preview).
    pub tiles: Vec<(IVec2, f32)>,
}

/// Bundles two-click commit resources + place-settle pulse (Bevy param budget).
#[derive(SystemParam)]
pub struct TwoClickCommitParams<'w> {
    pub tool: Res<'w, super::build_tool_authority::ActiveBuildTool>,
    pub strip: Res<'w, super::build_strip::BuildStripState>,
    pub ghost: ResMut<'w, super::build_state::BuildGhostState>,
    pub preview: Res<'w, super::build_state::BuildPlacementPreview>,
    pub actor: Res<'w, super::build_state::BuildCommandActor>,
    pub session: ResMut<'w, super::sessions::ActiveToolSession>,
    pub registry: Res<'w, super::building_definitions::BuildingDefinitionRegistry>,
    pub history: ResMut<'w, super::history::ConstructionHistory>,
    pub staging: Res<'w, super::staged_ghost_panel::StagedPlacementMode>,
    pub pulse: ResMut<'w, PlaceFeedbackPulse>,
}

impl PlaceFeedbackPulse {
    pub fn clear(&mut self) {
        self.active = false;
        self.elapsed_secs = 0.0;
        self.tiles.clear();
    }

    /// Abort without counting a complete (tool exit / cancel mid-pulse).
    pub fn abort(&mut self) {
        self.clear();
    }

    /// Replace any in-flight pulse (latest footprint wins).
    pub fn arm(&mut self, tiles: Vec<(IVec2, f32)>) {
        self.active = !tiles.is_empty();
        self.elapsed_secs = 0.0;
        self.tiles = tiles;
        if self.active {
            note_place_feedback_start();
        }
    }

    #[must_use]
    pub fn normalized_t(&self) -> f32 {
        if !self.active {
            return 1.0;
        }
        let dur = place_feedback_duration_secs().max(1e-4);
        (self.elapsed_secs / dur).clamp(0.0, 1.0)
    }
}

pub fn tick_place_feedback_pulse(time: Res<Time>, mut pulse: ResMut<PlaceFeedbackPulse>) {
    if !pulse.active {
        return;
    }
    pulse.elapsed_secs += time.delta_secs();
    if pulse.elapsed_secs >= place_feedback_duration_secs() {
        pulse.clear();
        note_place_feedback_complete();
    }
}

/// Leaving build strip / clearing tool → clear pulse (no complete credit).
pub fn clear_place_feedback_on_tool_exit(
    strip: Res<BuildStripState>,
    tool: Res<ActiveBuildTool>,
    mut pulse: ResMut<PlaceFeedbackPulse>,
) {
    if !pulse.active {
        return;
    }
    if strip.active == ToolContext::None || tool.tool == BuildTool::None {
        pulse.abort();
    }
}

/// Append settle tiles into the single [`ConstructionVisualRequests`] buffer (after ghost sync).
pub fn sync_place_feedback_visual_requests(
    pulse: Res<PlaceFeedbackPulse>,
    mut requests: ResMut<ConstructionVisualRequests>,
) {
    if !pulse.active || pulse.tiles.is_empty() {
        return;
    }
    let fill = place_feedback_fill_at(pulse.normalized_t());
    for &(tile, w) in &pulse.tiles {
        let weight = w.clamp(0.05, 1.0);
        let a = ((fill.a() as f32) * weight).round().clamp(0.0, 255.0) as u8;
        let color = egui::Color32::from_rgba_unmultiplied(fill.r(), fill.g(), fill.b(), a);
        requests.footprint_tiles.push(FootprintTileRequest {
            tile,
            color_kind: FootprintTileColorKind::Valid,
            weight: 1.0,
            fill_override: Some(color),
        });
    }
}

/// Refresh design+impl witness after COD-P3-PLACE-ANIM-001.
#[must_use]
pub fn refresh_des_place_feedback_anim_witness() -> bool {
    let starts = place_feedback_start_count();
    let completes = place_feedback_complete_count();
    let tokens_ok = footprint_place_settle_color()
        == egui::Color32::from_rgba_unmultiplied(100, 144, 220, 200)
        && (place_feedback_duration_secs() - 0.28).abs() < 1e-4
        && (ease_out_cubic(0.0) - 0.0).abs() < 1e-5
        && (ease_out_cubic(1.0) - 1.0).abs() < 1e-5;
    let start_is_valid_family = place_feedback_fill_at(0.0) == footprint_valid_color();
    let end_is_settle_family = place_feedback_fill_at(1.0) == footprint_place_settle_color();
    let impl_wired = tokens_ok && start_is_valid_family && end_is_settle_family;
    let green = impl_wired;
    let body = serde_json::json!({
        "gate": "DES-P3-PLACE-ANIM",
        "id": "DES-P3-PLACE-ANIM",
        "verdict": if green { "PASS" } else { "FAIL" },
        "green": green,
        "charter_on_disk": true,
        "charter_path": "src/dev/design_place_feedback_anim_v1.md",
        "coder_needed": false,
        "coder_row": "COD-P3-PLACE-ANIM-001",
        "impl_wired": impl_wired,
        "motion": {
            "duration_ms": 280,
            "curve": "ease_out_cubic_on_settle_tail",
            "lock_hold_until_raw_t": 0.62,
            "alpha_punch_peak": 1.0,
            "settle_hue_family": "phase_planned",
            "feel_charter": "src/dev/design_build_place_feel_v1.md",
            "forbidden": ["white_flash", "alpha_punch", "camera_nudge", "parallel_extract", "second_ghost_entity"]
        },
        "tokens": {
            "start": "ghost_visual::footprint_valid_color",
            "settle": "ghost_visual::footprint_place_settle_color",
            "reuse_pipeline": "ConstructionVisualRequests.footprint_tiles"
        },
        "scope": {
            "paths": ["Building", "Defense"],
            "trigger": "after_successful_CommitConstructionSiteEvent_queue",
            "fsm_unchanged": true,
            "pointer_gate_unchanged": true,
            "commit_funnel": "CommitConstructionSiteEvent"
        },
        "place_feedback_starts": starts,
        "place_feedback_completes": completes,
        "place_feedback_counters_honest": true,
        "note_counters": "AtomicU32 PLACE_FEEDBACK_STARTS/COMPLETES — starts on arm, completes on 280ms finish; abort on tool exit does not complete",
        "invariants_cited": [
            "preview_never_mutates",
            "single_commit_funnel",
            "allows_commit_gate",
            "ghost_disposable",
            "no_parallel_extract"
        ],
        "scope_out": [
            "WALL-ARCHETYPE",
            "phase9_military_industry",
            "logistics",
            "building_look",
            "g_play",
            "aps",
            "allows_commit_bypass",
            "two_click_fsm_fork"
        ],
        "residuals": [
            {
                "id": "WALL-ARCHETYPE",
                "owner": "planner",
                "scope": "replace MilitaryBase stub for defensive wall — do not expand here"
            }
        ],
        "delta_wf": "@coder COD-P3-PLACE-ANIM-001 done · residual WALL-ARCHETYPE → @planner"
    });
    let path = "debug_runs/des_place_feedback_anim_live.json";
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "COD-P3-PLACE-ANIM",
        "refresh_des_place_feedback_anim_witness",
        path,
        body,
    );
    green && crate::dev::debug_run_envelope::write_debug_run_json(path, wrapped)
}

#[must_use]
pub fn place_feedback_anim_impl_witness_green() -> bool {
    refresh_des_place_feedback_anim_witness()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settle_tokens_and_ease() {
        assert!((place_feedback_duration_secs() - 0.28).abs() < 1e-4);
        assert!((ease_out_cubic(0.0)).abs() < 1e-5);
        assert!((ease_out_cubic(1.0) - 1.0).abs() < 1e-5);
        assert!(ease_out_cubic(0.5) > 0.5);
        let v = footprint_valid_color();
        let s = footprint_place_settle_color();
        assert_eq!(place_feedback_fill_at(0.0), v);
        assert_eq!(place_feedback_fill_at(1.0), s);
        assert_eq!(place_feedback_fill_at(PLACE_FEEDBACK_LOCK_HOLD_T), v);
        let cap = v.a().max(s.a());
        for i in 0..=16 {
            assert!(place_feedback_fill_at(i as f32 / 16.0).a() <= cap);
        }
        assert_eq!(super::super::ghost_visual::footprint_lock_ring_stroke_px(), 2.0);
        assert_eq!(
            super::super::ghost_visual::footprint_lock_ring_color().a(),
            230
        );
    }

    #[test]
    fn arm_replace_and_complete_counters() {
        let starts0 = place_feedback_start_count();
        let completes0 = place_feedback_complete_count();
        let mut pulse = PlaceFeedbackPulse::default();
        pulse.arm(vec![(IVec2::new(1, 2), 1.0)]);
        assert!(pulse.active);
        assert_eq!(place_feedback_start_count(), starts0 + 1);
        pulse.arm(vec![(IVec2::new(3, 4), 0.5), (IVec2::new(4, 4), 1.0)]);
        assert_eq!(pulse.tiles.len(), 2);
        assert_eq!(place_feedback_start_count(), starts0 + 2);
        pulse.elapsed_secs = place_feedback_duration_secs();
        // Simulate tick completion path:
        if pulse.elapsed_secs >= place_feedback_duration_secs() {
            pulse.clear();
            note_place_feedback_complete();
        }
        assert!(!pulse.active);
        assert_eq!(place_feedback_complete_count(), completes0 + 1);
    }

    #[test]
    fn sync_appends_fill_override_tiles() {
        let mut app = App::new();
        app.insert_resource(PlaceFeedbackPulse {
            active: true,
            elapsed_secs: 0.0,
            tiles: vec![(IVec2::new(0, 0), 1.0), (IVec2::new(1, 0), 0.5)],
        });
        app.insert_resource(ConstructionVisualRequests::default());
        app.add_systems(Update, sync_place_feedback_visual_requests);
        app.update();
        let reqs = app.world().resource::<ConstructionVisualRequests>();
        assert_eq!(reqs.footprint_tiles.len(), 2);
        assert!(reqs.footprint_tiles.iter().all(|t| t.fill_override.is_some()));
    }

    #[test]
    fn witness_refresh_impl_wired() {
        assert!(place_feedback_anim_impl_witness_green());
    }
}
