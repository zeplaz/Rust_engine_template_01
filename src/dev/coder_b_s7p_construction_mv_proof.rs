//! **S7P-GRID-UX-001** + **CONSTRUCTION-MV-001** — lib witness bundle.

use serde_json::Value;

const INDUSTRIAL: &str = "debug_runs/industrial_activation_live.json";
const CONSTRUCTION: &str = "debug_runs/construction_stage_live.json";

fn repo_root() -> std::path::PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

fn read_json(rel: &str) -> Value {
    let path = repo_root().join(rel);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {rel}: {e}"))
}

/// Refresh industrial + construction witnesses for S7P grid UX and CONSTRUCTION-MV.
#[cfg(test)]
#[must_use]
pub fn refresh_s7p_grid_ux_and_construction_mv_witnesses() -> bool {
    assert!(
        crate::economy::activation::refresh_ind_e02_default_live_witness(),
        "IND-E02 seeds grid overload cluster"
    );
    assert!(
        crate::construction::refresh_construction_mv_001_live_witness(),
        "CONSTRUCTION-MV-001 sim writer"
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **S7P-GRID-UX-001** + **CONSTRUCTION-MV-001** closure bundle.
    #[test]
    fn s7p_grid_ux_and_construction_mv_001_bundle() {
        assert!(refresh_s7p_grid_ux_and_construction_mv_witnesses());

        let industrial = read_json(INDUSTRIAL);
        assert_eq!(
            industrial["s7p_grid_ux_001"]["toast_ui_wired"],
            Value::Bool(true)
        );
        assert_eq!(
            industrial["s7p_grid_ux_001"]["green"],
            Value::Bool(true)
        );

        let construction = read_json(CONSTRUCTION);
        assert_eq!(
            construction["construction_mv_001"]["green"],
            Value::Bool(true)
        );
        assert_eq!(
            construction["construction_mv_001"]["multiview_ghosts_wired"],
            Value::Bool(true)
        );
    }
}
