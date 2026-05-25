//! Full Bevy UI hierarchy trace + MAP_LAYOUT_CHAIN ancestry summary.
//!
//! Enable: `UI_LAYOUT_DEBUG=1` or `STAGE5_VERBOSE=1`.
//! Filter: `RUST_LOG=ui_layout_tree=info,ui_layout_tree::chain=info,ui_layout_tree::audit=warn`

use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, Node, UiSystems};
use bevy::window::PrimaryWindow;

use crate::gui::hud::viewport_authority_debug::viewport_authority_debug_enabled;
use crate::gui::in_game_hud::{
    SimulationCommandShellRoot, SimulationMapViewport, SimulationMapViewportDebug,
    SimulationMapViewportFill, SimulationMapViewportTrace,
};

pub const UI_LAYOUT_TREE_TARGET: &str = "ui_layout_tree";

/// Tag important HUD nodes for layout dumps (`hud_root`, `center_row`, `sim_map_fill`, …).
#[derive(Component, Clone, Copy, Debug)]
pub struct DebugLayoutTag(pub &'static str);

#[inline]
pub fn ui_layout_tree_debug_enabled() -> bool {
    static ENV: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENV.get_or_init(|| {
        std::env::var_os("UI_LAYOUT_DEBUG").is_some()
            || std::env::var_os("STAGE5_VERBOSE").is_some()
    })
}

fn layout_debug_active(session: Option<Res<crate::engine::FrameLayoutDebugSession>>) -> bool {
    ui_layout_tree_debug_enabled() || session.is_some_and(|s| s.active)
}

fn format_val(v: Val) -> String {
    match v {
        Val::Auto => "Auto".to_string(),
        Val::Px(x) => format!("{x:.1}px"),
        Val::Percent(p) => format!("{p:.1}%"),
        _ => format!("{v:?}"),
    }
}

/// Recursive dump of the command-shell subtree with `<<< SIM_VIEWPORT` marker on the map fill.
pub fn dump_ui_layout_tree(
    frame: Res<FrameCount>,
    layout_session: Option<Res<crate::engine::FrameLayoutDebugSession>>,
    shell_roots: Query<Entity, With<SimulationCommandShellRoot>>,
    q_children: Query<&Children>,
    q_node: Query<(&Node, &ComputedNode)>,
    q_name: Query<&Name>,
    q_tag: Query<&DebugLayoutTag>,
    q_fill: Query<Entity, With<SimulationMapViewportFill>>,
) {
    if !layout_debug_active(layout_session) || frame.0 % 60 != 0 {
        return;
    }
    let Ok(target) = q_fill.single() else {
        return;
    };

    for root in &shell_roots {
        info!(
            target: UI_LAYOUT_TREE_TARGET,
            frame = frame.0,
            root = ?root,
            target = ?target,
            "UI_LAYOUT_TREE"
        );
        recurse_tree(
            root,
            target,
            0,
            24,
            &q_children,
            &q_node,
            &q_name,
            &q_tag,
        );
    }
}

fn recurse_tree(
    entity: Entity,
    target: Entity,
    depth: usize,
    max_depth: usize,
    q_children: &Query<&Children>,
    q_node: &Query<(&Node, &ComputedNode)>,
    q_name: &Query<&Name>,
    q_tag: &Query<&DebugLayoutTag>,
) {
    if depth > max_depth {
        return;
    }

    let indent = "  ".repeat(depth);
    let marker = if entity == target {
        "<<< SIM_VIEWPORT"
    } else {
        ""
    };
    let name = q_name.get(entity).map(|n| n.as_str()).unwrap_or("unnamed");
    let tag = q_tag.get(entity).map(|t| t.0).unwrap_or("");

    if let Ok((style, computed)) = q_node.get(entity) {
        let size = computed.size();
        info!(
            target: UI_LAYOUT_TREE_TARGET,
            "{indent}{name} ({tag}) entity={entity:?} {marker} \
             size=({:.1},{:.1}) \
             width={} height={} \
             min=({},{}) max=({},{}) \
             flex_grow={:.2} flex_shrink={:.2} \
             flex_dir={:?} align_items={:?} align_self={:?} \
             justify={:?} overflow={:?}",
            size.x,
            size.y,
            format_val(style.width),
            format_val(style.height),
            format_val(style.min_width),
            format_val(style.min_height),
            format_val(style.max_width),
            format_val(style.max_height),
            style.flex_grow,
            style.flex_shrink,
            style.flex_direction,
            style.align_items,
            style.align_self,
            style.justify_content,
            style.overflow,
        );
    } else {
        info!(
            target: UI_LAYOUT_TREE_TARGET,
            "{indent}{name} ({tag}) entity={entity:?} {marker} (no Node/ComputedNode)",
        );
    }

    let Ok(children) = q_children.get(entity) else {
        return;
    };
    for child in children.iter() {
        recurse_tree(
            child,
            target,
            depth + 1,
            max_depth,
            q_children,
            q_node,
            q_name,
            &q_tag,
        );
    }
}

/// One-line ancestry chain: Window → shell → center_row → map fill → committed sizes.
pub fn dump_map_layout_chain(
    frame: Res<FrameCount>,
    layout_session: Option<Res<crate::engine::FrameLayoutDebugSession>>,
    win: Query<&Window, With<PrimaryWindow>>,
    sim: Res<SimulationMapViewport>,
    trace: Res<SimulationMapViewportTrace>,
    sim_dbg: Res<SimulationMapViewportDebug>,
    shell_roots: Query<Entity, With<SimulationCommandShellRoot>>,
    _q_children: Query<&Children>,
    q_node: Query<&ComputedNode>,
    q_name: Query<&Name>,
    q_tag: Query<&DebugLayoutTag>,
    q_fill: Query<Entity, With<SimulationMapViewportFill>>,
    child_of: Query<&ChildOf>,
) {
    if !layout_debug_active(layout_session) || frame.0 % 60 != 0 {
        return;
    }
    let Ok(window) = win.single() else {
        return;
    };
    let Ok(fill_e) = q_fill.single() else {
        return;
    };

    let win_wh = Vec2::new(window.width(), window.height());

    let mut chain: Vec<(String, Vec2)> = Vec::new();
    chain.push(("Window".to_string(), win_wh));

    if let Ok(shell) = shell_roots.single() {
        if let Ok(n) = q_node.get(shell) {
            chain.push(("RootHud".to_string(), n.size()));
        }
    }

    // Walk ancestors from map fill → shell (names/tags + computed sizes).
    let mut cursor = fill_e;
    let mut ancestors: Vec<(String, Vec2)> = Vec::new();
    for _ in 0..16 {
        if let Ok(n) = q_node.get(cursor) {
            let label = ancestor_label(cursor, &q_name, &q_tag);
            ancestors.push((label, n.size()));
        }
        let Some(parent) = child_of.get(cursor).ok().map(|c| c.parent()) else {
            break;
        };
        cursor = parent;
        if shell_roots.get(cursor).is_ok() {
            break;
        }
    }
    ancestors.reverse();
    chain.extend(ancestors);

    if let Ok(n) = q_node.get(fill_e) {
        chain.push(("MapFill".to_string(), n.size()));
    }
    chain.push(("Measured".to_string(), trace.measured_size));
    chain.push(("Committed".to_string(), trace.committed_size));
    if sim_dbg.solver_valid {
        chain.push((
            "Solver(SimMapFill)".to_string(),
            (sim_dbg.solver_max - sim_dbg.solver_min).max(Vec2::ZERO),
        ));
    }

    let mut lines = String::from("MAP_LAYOUT_CHAIN:");
    for (label, wh) in &chain {
        lines.push_str(&format!("\n  {label}: {:.0}x{:.0}", wh.x, wh.y));
    }
    if sim.valid {
        let committed = sim.logical_size();
        lines.push_str(&format!(
            "\n  CommittedResource: {:.0}x{:.0} last_commit={}",
            committed.x, committed.y, sim_dbg.last_commit
        ));
    }

    info!(target: "ui_layout_tree::chain", frame = frame.0, "{}", lines);

    if sim.valid {
        let committed = sim.logical_size();
        let map_wh = trace.measured_size;
        let dw = (committed.x - map_wh.x).abs();
        let dh = (committed.y - map_wh.y).abs();
        if dw > 8.0 || dh > 8.0 {
            warn!(
                target: "ui_layout_tree::chain",
                frame = frame.0,
                committed_w = committed.x,
                committed_h = committed.y,
                map_fill_w = map_wh.x,
                map_fill_h = map_wh.y,
                delta_w = dw,
                delta_h = dh,
                last_commit = sim_dbg.last_commit,
                "COMMITTED_VS_MAPFILL_MISMATCH (camera must use sim_map_fill, not hud_root/solver)"
            );
        }
    }

    if let Ok(shell) = shell_roots.single() {
        if let Ok(shell_size) = q_node.get(shell).map(|n| n.size()) {
            let win_gap = (win_wh - shell_size).length();
            if win_gap > 16.0 {
                warn!(
                    target: "ui_layout_tree::chain",
                    frame = frame.0,
                    window = ?win_wh,
                    hud_root = ?shell_size,
                    "HUD_ROOT_NOT_FULL_WINDOW (shell % sizing parent may not be swapchain viewport)"
                );
            }
        }
    }

    // Highlight first ancestor whose height shrinks vs parent (flex collapse suspect).
    for i in 1..chain.len() {
        let (child_label, child_wh) = &chain[i];
        let (parent_label, parent_wh) = &chain[i - 1];
        if parent_wh.y > 64.0 && child_wh.y < parent_wh.y * 0.85 {
            warn!(
                target: "ui_layout_tree::chain",
                frame = frame.0,
                parent = parent_label,
                parent_h = parent_wh.y,
                child = child_label,
                child_h = child_wh.y,
                "LAYOUT_HEIGHT_COLLAPSE suspect ancestor"
            );
        }
    }
}

fn ancestor_label(entity: Entity, q_name: &Query<&Name>, q_tag: &Query<&DebugLayoutTag>) -> String {
    if let Ok(t) = q_tag.get(entity) {
        return t.0.to_string();
    }
    q_name
        .get(entity)
        .map(|n| n.as_str().to_string())
        .unwrap_or_else(|_| format!("{entity:?}"))
}

/// ±1px invariant: committed logical hole must match camera physical scissor (after scale).
pub fn audit_camera_scissor_matches_committed(
    sim: Res<SimulationMapViewport>,
    cam: Query<&Camera, With<crate::gui::MainWorldCamera>>,
    win: Query<&Window, With<PrimaryWindow>>,
    ortho: Res<crate::gui::MainWorldCameraOrthoTrace>,
) {
    if !viewport_authority_debug_enabled() {
        return;
    }
    if !sim.valid || !ortho.using_hole {
        return;
    }
    let Ok(camera) = cam.single() else {
        return;
    };
    let Some(vp) = &camera.viewport else {
        warn!(
            target: "ui_layout_tree::audit",
            "CAMERA_SCISSOR_AUTHORITY_LEAK render_hole=true but camera.viewport=None"
        );
        return;
    };
    let Ok(w) = win.single() else {
        return;
    };
    let scale = w.scale_factor().max(1e-6);
    let committed_w = ((sim.max.x - sim.min.x) * scale).round() as i32;
    let committed_h = ((sim.max.y - sim.min.y) * scale).round() as i32;
    let scissor_w = vp.physical_size.x as i32;
    let scissor_h = vp.physical_size.y as i32;

    let dw = committed_w.abs_diff(scissor_w);
    let dh = committed_h.abs_diff(scissor_h);
    if dw > 1 || dh > 1 {
        warn!(
            target: "ui_layout_tree::audit",
            committed_w,
            committed_h,
            scissor_w,
            scissor_h,
            delta_w = dw,
            delta_h = dh,
            sim_min = ?sim.min,
            sim_max = ?sim.max,
            "CAMERA_SCISSOR_AUTHORITY_LEAK committed vs scissor (±1px)"
        );
    }
}

pub struct UiLayoutTreeDebugPlugin;

impl Plugin for UiLayoutTreeDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                dump_ui_layout_tree,
                dump_map_layout_chain,
            )
                .chain()
                .after(UiSystems::Layout)
                .run_if(crate::gui::in_simulation_or_editor),
        )
        .add_systems(
            PostUpdate,
            audit_camera_scissor_matches_committed
                .after(crate::gui::map_camera::sync_main_world_camera_viewport_and_projection)
                .run_if(crate::gui::in_simulation_or_editor_map),
        );
    }
}
