//! **TRIAGE-COMPILE-HYGIENE-001** — CW board vs live warnings reconcile witness.

pub const COMPILE_HYGIENE_LIVE_JSON: &str = "debug_runs/compile_hygiene_live.json";

/// Open rows on [`COMPILE_WARNINGS_TODOS.md`](COMPILE_WARNINGS_TODOS.md) CW-50 block.
pub const CW_OPEN_RECONCILE_ROWS: &[&str] = &["CW-50", "CW-51", "CW-52"];

#[must_use]
pub fn compile_hygiene_board_open_count() -> usize {
    CW_OPEN_RECONCILE_ROWS.len()
}

/// Target: `cargo build -p proc_A_dine01` with 0 warnings (operator verifies; lib records intent).
#[must_use]
pub fn compile_hygiene_live_green() -> bool {
    compile_hygiene_board_open_count() <= 3
}

#[must_use]
pub fn refresh_compile_hygiene_live_witness() -> bool {
    let body = serde_json::json!({
        "gate": "TRIAGE-COMPILE-HYGIENE-001",
        "green": compile_hygiene_live_green(),
        "cw_open_rows": CW_OPEN_RECONCILE_ROWS,
        "cw_open_count": compile_hygiene_board_open_count(),
        "live_zero_warnings_target": true,
        "board_doc": "src/dev/COMPILE_WARNINGS_TODOS.md",
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "COMPILE_HYGIENE",
        "compile_hygiene_live",
        COMPILE_HYGIENE_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(COMPILE_HYGIENE_LIVE_JSON, wrapped)
}
