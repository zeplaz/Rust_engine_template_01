//! **UI-W3-P6-001** — @coder B: shell perf + multiview witness rollup ([`ui_phase6_shell_perf_multiview_plan_v1.md`](ui_phase6_shell_perf_multiview_plan_v1.md)).

use std::path::PathBuf;

use serde_json::Value;

const UI_SHELL: &str = "debug_runs/ui_shell_migration_live.json";
const INFRA: &str = "debug_runs/infrastructure_view_isolation_live.json";
const STAGE6: &str = "debug_runs/stage6_virtualization_live.json";
const MINIMAP: &str = "debug_runs/minimap_compositor_live.json";

const P6_SHELL: &[&str] = &[
    "/ui_w3_p6_001/green",
    "/ui_w3_p6_001/shell_perf_green",
    "/phase2b_closed",
    "/egui_pass_count_in_sim",
    "/phase5/pause_menu_bevy",
    "/ui_p5_pause_001_green",
];

const P6_MULTIVIEW: &[(&str, &[&str])] = &[
    (INFRA, &["/infrastructure_view_isolation_green", "/vm_08/overlay_masks_aligned"]),
    (STAGE6, &["/stage6_virtualization_green"]),
    (MINIMAP, &["/composite_path"]),
];

fn repo_root() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_json(rel: &str) -> Value {
    let path = repo_root().join(rel);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {rel}: {e}"))
}

fn pointer_bool(v: &Value, ptr: &str) -> bool {
    v.pointer(ptr)
        .and_then(|x| x.as_bool())
        .unwrap_or_else(|| panic!("missing or non-bool {ptr}"))
}

fn pointer_str(v: &Value, ptr: &str) -> String {
    v.pointer(ptr)
        .and_then(|x| x.as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| panic!("missing or non-string {ptr}"))
}

fn pointer_u64(v: &Value, ptr: &str) -> u64 {
    v.pointer(ptr)
        .and_then(|x| x.as_u64())
        .unwrap_or_else(|| panic!("missing or non-number {ptr}"))
}

/// **UI-W3-P6-001** — full rollup across shell + infra + stage6 + minimap (lib paths).
#[must_use]
pub fn ui_w3_p6_001_cross_file_green() -> bool {
    let shell = read_json(UI_SHELL);
    if !pointer_bool(&shell, "/ui_w3_p6_001/shell_perf_green") {
        return false;
    }
    if pointer_u64(&shell, "/egui_pass_count_in_sim") != 0 {
        return false;
    }
    let infra = read_json(INFRA);
    if !pointer_bool(&infra, "/infrastructure_view_isolation_green") {
        return false;
    }
    let stage6 = read_json(STAGE6);
    if !pointer_bool(&stage6, "/stage6_virtualization_green") {
        return false;
    }
    let minimap = read_json(MINIMAP);
    pointer_str(&minimap, "/composite_path") == "GpuCompute"
}

/// Single writer: witness bundle + shell `ui_w3_p6_001` block.
pub fn refresh_ui_w3_p6_001_live_witness() -> bool {
    use super::coder_b_ui_w3_witness_proof::refresh_ui_w3_witness_001_live_witness;

    assert!(
        refresh_ui_w3_witness_001_live_witness(),
        "UI-W3-P6-001: prerequisite witness refresh"
    );
    assert!(ui_w3_p6_001_cross_file_green(), "UI-W3-P6-001 cross-file rollup");
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **UI-W3-P6-001** — shell perf + multiview isolation witness bundle.
    #[test]
    fn coder_b_ui_w3_p6_001_lib_bundle() {
        assert!(refresh_ui_w3_p6_001_live_witness());

        let shell = read_json(UI_SHELL);
        for ptr in P6_SHELL {
            if ptr.ends_with("/egui_pass_count_in_sim") {
                assert_eq!(pointer_u64(&shell, ptr), 0, "{UI_SHELL} {ptr}");
            } else {
                assert!(pointer_bool(&shell, ptr), "{UI_SHELL} {ptr} must be true");
            }
        }

        for (path, pointers) in P6_MULTIVIEW {
            let v = read_json(path);
            for ptr in *pointers {
                if *ptr == "/composite_path" {
                    assert_eq!(pointer_str(&v, ptr), "GpuCompute", "{path} GpuCompute");
                } else {
                    assert!(pointer_bool(&v, ptr), "{path} {ptr} must be true");
                }
            }
        }
    }
}
