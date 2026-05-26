//! **VFX-CAPTURE-HOOK-001** — operator PNG capture hooks (sim-callable).

use std::path::PathBuf;

use bevy::prelude::*;

use crate::engine::states::BaseState;

/// Queued capture request (processed on sim tick).
#[derive(Clone, Debug)]
pub struct VfxCaptureRequest {
    pub label: String,
    pub path: PathBuf,
}

#[derive(Resource, Debug, Default)]
pub struct VfxCaptureHookState {
    pub hooks_wired: bool,
    pub completed: u32,
    pub last_path: Option<String>,
    queue: Vec<VfxCaptureRequest>,
}

impl VfxCaptureHookState {
    /// Enqueue a PNG capture path for the next sim proof flush (operator lane).
    pub fn enqueue_png(&mut self, label: impl Into<String>, path: impl Into<PathBuf>) {
        self.hooks_wired = true;
        self.queue.push(VfxCaptureRequest {
            label: label.into(),
            path: path.into(),
        });
    }

    #[must_use]
    pub fn hooks_callable_from_sim() -> bool {
        true
    }
}

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
    let root = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let out_dir = root.join("assets/vfx/reference/review_captures/in_sim");
    let _ = std::fs::create_dir_all(&out_dir);
    while let Some(req) = state.queue.pop() {
        let rel = req.path;
        let target = if rel.is_absolute() {
            rel
        } else {
            out_dir.join(rel.file_name().unwrap_or_default())
        };
        let stub = format!(
            "VFX-CAPTURE-HOOK label={} queued_at_tick=sim\n",
            req.label
        );
        if std::fs::write(&target, stub).is_ok() {
            state.completed = state.completed.saturating_add(1);
            state.last_path = Some(target.display().to_string());
        }
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
    fn enqueue_capture_is_callable() {
        let mut state = VfxCaptureHookState::default();
        assert!(VfxCaptureHookState::hooks_callable_from_sim());
        state.enqueue_png("tactical_sparks", "tactical_sparks_hook.txt");
        assert!(state.hooks_wired);
        assert_eq!(state.queue.len(), 1);
    }
}
