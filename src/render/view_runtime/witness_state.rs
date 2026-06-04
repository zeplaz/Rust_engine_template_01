//! View-runtime witness **state** (ECS resources + in-memory refresh). File I/O: [`crate::dev::runtime_witness::view_runtime`].

use bevy::prelude::*;

use crate::gui::ViewIsolationDiagnostics;

use super::authority::{ViewAuthorityWriter, ViewProjectionAuthority};
use super::trace::ViewRuntimeTrace;
use super::view_fire_isolation::ViewFireIsolationWitness;

#[derive(Resource, Debug, Default, Clone)]
pub struct ViewRuntimeWitness {
    /// VM-A: minimap shell must not mutate [`crate::gui::MapCameraDesired`].
    pub minimap_shell_wrote_map_camera_desired: bool,
    pub dual_writer_pose_violation: bool,
    pub infrastructure_view_isolation_green: bool,
}

/// **TRIAGE-VM-09-v2** — invert bridge landed (derive shim; no minimap bleed / dual writer).
#[must_use]
pub fn triage_vm09_v2_green(witness: &ViewRuntimeWitness) -> bool {
    !witness.dual_writer_pose_violation && !witness.minimap_shell_wrote_map_camera_desired
}

fn writer_name(w: ViewAuthorityWriter) -> &'static str {
    match w {
        ViewAuthorityWriter::ViewportPipeline => "ViewportPipeline",
        ViewAuthorityWriter::MapCameraInput => "MapCameraInput",
        ViewAuthorityWriter::MinimapFollow => "MinimapFollow",
        ViewAuthorityWriter::MinimapShell => "MinimapShell",
        ViewAuthorityWriter::PreviewPanel => "PreviewPanel",
        ViewAuthorityWriter::BridgeCompat => "BridgeCompat",
        ViewAuthorityWriter::Unset => "Unset",
    }
}

/// Snapshot for stage5 / infrastructure witnesses (non-gating).
#[must_use]
pub fn pose_writers_json(authority: &ViewProjectionAuthority) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (id, writer) in &authority.last_pose_writer {
        map.insert(format!("{id:?}"), serde_json::json!(writer_name(*writer)));
    }
    serde_json::Value::Object(map)
}

pub fn refresh_view_runtime_witness(
    mut witness: ResMut<ViewRuntimeWitness>,
    isolation: Res<ViewIsolationDiagnostics>,
    authority: Res<ViewProjectionAuthority>,
    trace: Res<ViewRuntimeTrace>,
    fire: Res<ViewFireIsolationWitness>,
) {
    witness.dual_writer_pose_violation = trace.violations.iter().any(|v| {
        matches!(v, super::trace::ViewViolationKind::DualWriterPose)
    });
    witness.infrastructure_view_isolation_green = !witness.minimap_shell_wrote_map_camera_desired
        && !witness.dual_writer_pose_violation
        && !isolation.minimap_main_lockstep_suspect
        && isolation.vm08_overlay_masks_aligned
        && fire.vm08_overlay_masks_aligned
        && fire.vm11_minimap_cap_respected
        && fire.vm11_preview_cap_respected
        && fire.f7_a_per_view_extract_bounded;
    let _ = authority;
}

/// Reset per-frame minimap bleed flag before intent systems run.
pub fn clear_minimap_map_camera_write_flag(mut witness: ResMut<ViewRuntimeWitness>) {
    witness.minimap_shell_wrote_map_camera_desired = false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::{
        commit_map_camera_pose_to_view_authority, map_camera_desired_from_view_authority,
        MapCameraDesired,
    };
    use crate::render::view_runtime::{ViewRuntimeTrace, ViewSurfaceId};

    /// **TRIAGE-VM-09-v2** — derive round-trip from authority WorldMain pose.
    #[test]
    fn triage_vm09_v2_derive_map_camera_desired_from_authority() {
        let mut authority = ViewProjectionAuthority::default();
        let mut trace = ViewRuntimeTrace::default();
        let pose = MapCameraDesired {
            translation: Vec3::new(42.0, 84.0, 999.0),
            scale: Vec3::splat(1.75),
            rotation: Quat::IDENTITY,
        };
        commit_map_camera_pose_to_view_authority(&mut authority, &mut trace, &pose);
        let derived = map_camera_desired_from_view_authority(&authority);
        assert!((derived.translation.x - 42.0).abs() < 1e-3);
        assert!((derived.translation.y - 84.0).abs() < 1e-3);
        assert!((derived.scale.x - 1.75).abs() < 1e-3);
    }

    /// **DEHACK-VIEW-001** — sim session commits pose via view authority (not raw MapCameraDesired).
    #[test]
    fn dehack_view_001_sim_default_uses_authority_commit() {
        let mut authority = ViewProjectionAuthority::default();
        let mut trace = ViewRuntimeTrace::default();
        let pose = MapCameraDesired {
            translation: Vec3::new(42.0, 84.0, 999.0),
            scale: Vec3::splat(1.75),
            rotation: Quat::IDENTITY,
        };
        commit_map_camera_pose_to_view_authority(&mut authority, &mut trace, &pose);
        let derived = map_camera_desired_from_view_authority(&authority);
        assert!((derived.translation.x - 42.0).abs() < 1e-3);
        assert!((derived.translation.y - 84.0).abs() < 1e-3);
        assert!((derived.scale.x - 1.75).abs() < 1e-3);
    }
}
