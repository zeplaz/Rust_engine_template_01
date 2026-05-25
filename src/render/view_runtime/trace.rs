use bevy::prelude::*;

use super::authority::ViewAuthorityWriter;
use super::ids::ViewSurfaceId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewViolationKind {
    DualWriterPose,
    DualWriterRender,
    MinimapMainLockstep,
    PreviewWorldCommit,
    ExtentMismatch,
}

#[derive(Clone, Debug)]
pub struct ViewRuntimeTraceEntry {
    pub frame: u64,
    pub surface: ViewSurfaceId,
    pub writer: ViewAuthorityWriter,
    pub note: &'static str,
}

/// Ring buffer of commits (VM-A diagnostics; populated when `VIEW_RUNTIME_AUDIT=1`).
#[derive(Resource, Default, Debug)]
pub struct ViewRuntimeTrace {
    pub enabled: bool,
    pub frame: u64,
    pub entries: Vec<ViewRuntimeTraceEntry>,
    pub violations: Vec<ViewViolationKind>,
}

impl ViewRuntimeTrace {
    pub const MAX_ENTRIES: usize = 256;

    pub fn record(&mut self, surface: ViewSurfaceId, writer: ViewAuthorityWriter, note: &'static str) {
        if !self.enabled {
            return;
        }
        if self.entries.len() >= Self::MAX_ENTRIES {
            self.entries.remove(0);
        }
        self.entries.push(ViewRuntimeTraceEntry {
            frame: self.frame,
            surface,
            writer,
            note,
        });
    }

    pub fn push_violation(&mut self, kind: ViewViolationKind) {
        if self.enabled && !self.violations.contains(&kind) {
            self.violations.push(kind);
        }
    }
}

pub fn advance_view_runtime_trace_frame(mut trace: ResMut<ViewRuntimeTrace>) {
    trace.frame = trace.frame.saturating_add(1);
    if trace.enabled && trace.entries.len() > ViewRuntimeTrace::MAX_ENTRIES {
        trace.entries.clear();
    }
}
