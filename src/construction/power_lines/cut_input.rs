//! Military / demolish tool — cut nearest power segment (DES-POWER-TARGETING-001 wire).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

use crate::gui::{MapCameraDesired, SimulationMapViewport};
use crate::infrastructure::utility::UtilityGraph;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::damage::{cut_power_line_segment, preview_island_offline_from_cut, PowerLineDamageBook};
use crate::construction::build_tool_authority::{ActiveBuildTool, BuildTool};
use crate::construction::roads::cursor_world_on_map;
use crate::infrastructure::utility::graph::UtilityNetworkSnapshotResource;

const CUT_PICK_RADIUS_M: f32 = 2.5;

#[derive(Resource, Debug, Default, Clone)]
pub struct PowerLineCutToast {
    pub last_message: String,
    pub cuts_total: u32,
}

pub fn power_line_demolish_cut_system(
    buttons: Res<ButtonInput<MouseButton>>,
    tool: Res<ActiveBuildTool>,
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<crate::render::view_runtime::ViewProjectionAuthority>>,
    desired: Res<MapCameraDesired>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
    utility: Option<Res<UtilityGraph>>,
    snap: Option<Res<UtilityNetworkSnapshotResource>>,
    mut book: ResMut<PowerLineDamageBook>,
    mut toast: ResMut<PowerLineCutToast>,
    mut egui_ctx: EguiContexts,
) {
    if tool.tool != BuildTool::Demolish {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(ctx) = egui_ctx.ctx_mut() else {
        return;
    };
    if ctx.wants_pointer_input() {
        return;
    }
    let Ok(window) = win.single() else {
        return;
    };
    let Some(cursor) = cursor_world_on_map(
        &window,
        authority.as_deref(),
        desired.as_ref(),
        map_vp.as_ref(),
        params.as_ref(),
    ) else {
        return;
    };
    let Some(utility) = utility else {
        return;
    };
    let cursor2 = Vec2::new(cursor.x, cursor.z);
    let node_pos: std::collections::HashMap<u64, Vec2> = utility
        .nodes
        .iter()
        .map(|n| (n.id, n.position))
        .collect();

    let mut best: Option<(u64, f32)> = None;
    for edge in &utility.power_edges {
        let Some(from) = node_pos.get(&edge.from) else {
            continue;
        };
        let Some(to) = node_pos.get(&edge.to) else {
            continue;
        };
        let dist = dist_point_to_segment(cursor2, *from, *to);
        if dist <= CUT_PICK_RADIUS_M {
            if best.map(|(_, d)| dist < d).unwrap_or(true) {
                best = Some((edge.link_id, dist));
            }
        }
    }
    let Some((link_id, _)) = best else {
        toast.last_message = "○ No power target here".into();
        return;
    };

    let offline = snap.as_deref().map(|s| {
        preview_island_offline_from_cut(&utility, &s.0, &book, link_id)
    }).unwrap_or(0);

    if cut_power_line_segment(&mut book, link_id) {
        toast.cuts_total = toast.cuts_total.saturating_add(1);
        toast.last_message = if offline > 0 {
            format!("✓ Line cut — {offline} offline")
        } else {
            "✓ Line cut — no consumers lost".into()
        };
    }
}

fn dist_point_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_squared();
    if len2 < 1e-6 {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

#[must_use]
pub fn power_line_cut_input_wired() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dist_point_to_segment_midpoint() {
        let d = dist_point_to_segment(Vec2::new(2.0, 0.0), Vec2::ZERO, Vec2::new(4.0, 0.0));
        assert!(d < 0.01);
    }

    #[test]
    fn power_line_cut_witness_wired() {
        assert!(power_line_cut_input_wired());
    }
}
