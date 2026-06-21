//! Disk-only [`PerfScope`](crate::render::PerfScope) accumulation for `--test` instrumentation runs.

use std::sync::Mutex;

use serde_json::{json, Value};

static SCOPES: Mutex<Vec<(&'static str, f32)>> = Mutex::new(Vec::new());

pub fn record_perf_scope(label: &'static str, ms: f32) {
    if !crate::dev::test_run_instrumentation::instrumentation_active() || !ms.is_finite() {
        return;
    }
    if let Ok(mut scopes) = SCOPES.lock() {
        scopes.push((label, ms.max(0.0)));
    }
}

/// Drain scopes recorded this frame (sorted descending by ms).
#[must_use]
pub fn take_perf_scopes_json() -> Value {
    let mut scopes = SCOPES
        .lock()
        .ok()
        .map(|mut guard| std::mem::take(&mut *guard))
        .unwrap_or_default();
    scopes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    json!(scopes
        .into_iter()
        .take(32)
        .map(|(label, ms)| json!({ "label": label, "ms": ms }))
        .collect::<Vec<_>>())
}
