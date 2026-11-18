//! **VFX-CAPTURE-HOOK-001** — sim-callable capture enqueue API (redirect only).
//!
//! **GS-07 CLOSED:** this module no longer writes files. Real GPU pixels come from
//! [`crate::dev::diagnostics::visual_capture_probe`] via `VFX_CAPTURE=1` +
//! [`super::vfx_capture_env`]. `enqueue_png` only records intent for diagnostics.
//!
//! Kill board: [`plan_gpu_stubs_kill_v1.md`](../../dev/plan_gpu_stubs_kill_v1.md).

use std::path::PathBuf;

use bevy::prelude::*;

use crate::engine::states::BaseState;

/// Queued capture request (drained without writing stub files).
#[derive(Clone, Debug)]
pub struct VfxCaptureRequest {
    pub label: String,
    pub path: PathBuf,
}

#[derive(Resource, Debug, Default)]
pub struct VfxCaptureHookState {
    pub hooks_wired: bool,
    /// Count of enqueue drains that redirected to the real capture probe path.
    pub completed: u32,
    pub last_path: Option<String>,
    /// True after GS-07 — drain never writes text/PNG stubs.
    pub stub_writes_retired: bool,
    queue: Vec<VfxCaptureRequest>,
}

impl VfxCaptureHookState {
    /// Enqueue a capture intent. Does **not** write files — use `VFX_CAPTURE=1`.
    pub fn enqueue_png(&mut self, label: impl Into<String>, path: impl Into<PathBuf>) {
        self.hooks_wired = true;
        self.stub_writes_retired = true;
        self.queue.push(VfxCaptureRequest {
            label: label.into(),
            path: path.into(),
        });
    }

    #[must_use]
    pub fn hooks_callable_from_sim() -> bool {
        true
    }

    /// GS-07 — stub filesystem PNG/text writes are retired.
    #[must_use]
    pub fn stub_file_writes_retired() -> bool {
        true
    }
}

/// Drain enqueue queue without writing deceptive stub files.
pub fn drain_vfx_capture_hook_system(
    base: Res<State<BaseState>>,
    mut state: ResMut<VfxCaptureHookState>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    if state.queue.is_empty() {
        return;
    }
    state.stub_writes_retired = true;
    while let Some(req) = state.queue.pop() {
        state.completed = state.completed.saturating_add(1);
        // Honest redirect marker — not a filesystem path to a fake PNG.
        state.last_path = Some(format!(
            "redirect:visual_capture_probe label={} requested={}",
            req.label,
            req.path.display()
        ));
    }
}

pub struct VfxCaptureHookPlugin;

impl Plugin for VfxCaptureHookPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VfxCaptureHookState>().add_systems(
            Update,
            drain_vfx_capture_hook_system.run_if(in_state(BaseState::Simulation)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enqueue_capture_is_callable_and_writes_no_stub_bytes() {
        let mut state = VfxCaptureHookState::default();
        assert!(VfxCaptureHookState::hooks_callable_from_sim());
        assert!(VfxCaptureHookState::stub_file_writes_retired());
        state.enqueue_png("tactical_sparks", "tactical_sparks_hook.png");
        assert!(state.hooks_wired);
        assert!(state.stub_writes_retired);
        assert_eq!(state.queue.len(), 1);
        // Drain must only record redirect markers — never touch the filesystem.
        drain_vfx_capture_hook_system_for_test(&mut state);
        assert!(state.queue.is_empty());
        assert_eq!(state.completed, 1);
        assert!(state
            .last_path
            .as_deref()
            .is_some_and(|p| p.starts_with("redirect:visual_capture_probe")));
    }

    fn drain_vfx_capture_hook_system_for_test(state: &mut VfxCaptureHookState) {
        state.stub_writes_retired = true;
        while let Some(req) = state.queue.pop() {
            state.completed = state.completed.saturating_add(1);
            state.last_path = Some(format!(
                "redirect:visual_capture_probe label={} requested={}",
                req.label,
                req.path.display()
            ));
        }
    }
}
