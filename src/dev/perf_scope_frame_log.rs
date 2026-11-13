//! Per-frame [`PerfScope`](crate::render::PerfScope) totals, kept for the hitch detector.
//!
//! Always on (release too) and allocation-free: a fixed table of the costliest scope labels this
//! frame, summed per label. [`crate::render::probes::frame_hitch`] drains it once per frame and
//! names the top entries when a frame is flagged as a hitch.

use std::sync::Mutex;

/// Distinct scope labels kept per frame; extra labels replace the cheapest entry when costlier.
pub const FRAME_SCOPE_SLOTS: usize = 16;

#[derive(Clone, Copy)]
pub struct FrameScopes {
    pub entries: [(&'static str, f32); FRAME_SCOPE_SLOTS],
    pub len: usize,
}

impl FrameScopes {
    pub const EMPTY: Self = Self {
        entries: [("", 0.0); FRAME_SCOPE_SLOTS],
        len: 0,
    };

    fn add(&mut self, label: &'static str, ms: f32) {
        if let Some(e) = self.entries[..self.len].iter_mut().find(|e| e.0 == label) {
            e.1 += ms;
            return;
        }
        if self.len < FRAME_SCOPE_SLOTS {
            self.entries[self.len] = (label, ms);
            self.len += 1;
            return;
        }
        if let Some(min) = self.entries.iter_mut().min_by(|a, b| a.1.total_cmp(&b.1)) {
            if ms > min.1 {
                *min = (label, ms);
            }
        }
    }

    /// The `N` costliest labels, descending; unused slots are `("", 0.0)`.
    #[must_use]
    pub fn top<const N: usize>(&self) -> [(&'static str, f32); N] {
        let mut sorted = self.entries;
        sorted[..self.len].sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
        let mut out = [("", 0.0); N];
        for (o, e) in out.iter_mut().zip(sorted[..self.len].iter()) {
            *o = *e;
        }
        out
    }
}

static FRAME: Mutex<FrameScopes> = Mutex::new(FrameScopes::EMPTY);

pub fn record_perf_scope(label: &'static str, ms: f32) {
    if !ms.is_finite() {
        return;
    }
    if let Ok(mut frame) = FRAME.lock() {
        frame.add(label, ms.max(0.0));
    }
}

/// Take this frame's scope totals and start the next frame empty.
#[must_use]
pub fn take_frame_scopes() -> FrameScopes {
    FRAME
        .lock()
        .map(|mut f| std::mem::replace(&mut *f, FrameScopes::EMPTY))
        .unwrap_or(FrameScopes::EMPTY)
}
