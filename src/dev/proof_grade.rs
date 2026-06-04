//! **G-PROOF-01** — witness proof grades (PLAY-TRUTH-002).
//!
//! `VisualCapture` (`--test visual` / `full_capture_active`) must not use witness shortcuts,
//! `patch_*_witness*`, or `qualified_close` as a green substitute.

/// How a witness JSON row was produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProofGrade {
    /// Lib-only fixture writers (`refresh_*_live_witness`, projection fixtures).
    LibFixture,
    /// Headless sim / harness without full visual capture proof commit.
    HeadlessSim,
    /// Live `--test visual` proof commit (`full_capture_active`).
    VisualCapture,
}

impl ProofGrade {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LibFixture => "lib_fixture",
            Self::HeadlessSim => "headless_sim",
            Self::VisualCapture => "visual_capture",
        }
    }

    /// **G-PROOF-01:** atomic LOG-* shortcuts and `patch_*_witness_for_play_proof` are lib/harness only.
    #[must_use]
    pub const fn allows_witness_shortcuts(self) -> bool {
        !matches!(self, Self::VisualCapture)
    }

    /// **G-PROOF-01:** `qualified_close` may close LOG-E01 on lib fixture only.
    #[must_use]
    pub const fn allows_qualified_close_green(self) -> bool {
        matches!(self, Self::LibFixture)
    }

    #[must_use]
    pub const fn from_full_capture_active(full_capture_active: bool) -> Self {
        if full_capture_active {
            Self::VisualCapture
        } else {
            Self::HeadlessSim
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn proof_grade_visual_capture_disallows_shortcuts() {
        assert!(!ProofGrade::VisualCapture.allows_witness_shortcuts());
        assert!(!ProofGrade::VisualCapture.allows_qualified_close_green());
        assert!(ProofGrade::LibFixture.allows_witness_shortcuts());
        assert!(ProofGrade::LibFixture.allows_qualified_close_green());
    }

    /// **PLAY-TRUTH-002** grep gate — visual capture sources must not call logistics shortcuts.
    #[test]
    fn proof_grade_visual_capture_has_no_witness_shortcuts() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let visual_lane_sources = [
            "src/render/stage5_full_app_harness.rs",
            "src/engine/test_harness.rs",
            "src/economy/logistics/mod.rs",
            "src/economy/logistics/witness.rs",
        ];
        let forbidden = [
            "apply_s7p_logistics_throughput_witness_shortcut",
            "patch_s7p_logistics_throughput_witness_for_play_proof",
        ];
        for rel in visual_lane_sources {
            let path = root.join(rel);
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {rel}: {e}"));
            for needle in forbidden {
                assert!(
                    !content.contains(needle),
                    "G-PROOF-01: {rel} must not reference `{needle}` (VisualCapture lane)"
                );
            }
        }
    }

    /// **DEHACK-ENG-001** — harness control types are not re-exported on `crate::engine::*`.
    #[test]
    fn dehack_eng_001_no_harness_on_engine_root() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mod_rs = std::fs::read_to_string(root.join("src/engine/mod.rs"))
            .expect("read engine/mod.rs");
        let forbidden = [
            "TestHarnessPlugin",
            "TestWorldHarness",
            "DebugQuickWorldGenPending",
            "arm_debug_quick_world_gen",
            "TestHarnessMenuPlugin",
            "TestHarnessStatePlugin",
        ];
        for needle in forbidden {
            assert!(
                !mod_rs.contains(needle),
                "DEHACK-ENG-001: engine/mod.rs must not re-export `{needle}` — use `engine::test_harness::`"
            );
        }
        assert!(
            mod_rs.contains("pub use test_harness::ActiveTestScene"),
            "ActiveTestScene remains the narrow sim/test-scene marker export"
        );
    }

    /// **DEHACK-RENDER-001** — `refresh_*_live_witness` helpers are not re-exported on `crate::render::*`.
    #[test]
    fn dehack_render_001_no_refresh_live_witness_on_render_root() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mod_rs = std::fs::read_to_string(root.join("src/render/mod.rs"))
            .expect("read render/mod.rs");
        let forbidden = [
            "refresh_log_e01_",
            "refresh_p2_fire_",
            "refresh_wc_d04_",
            "refresh_infra_slice3_",
            "refresh_infrastructure_view_",
            "refresh_fire_streaming_",
        ];
        for needle in forbidden {
            assert!(
                !mod_rs.contains(needle),
                "DEHACK-RENDER-001: render/mod.rs must not re-export `{needle}` — use runtime_witness or submodule path in lib tests"
            );
        }
    }

    /// **INFRA-E0-002** — tile `TerrainFeatures.road/track` and new sim reads outside allowlist.
    #[test]
    fn infra_e0_002_tile_transport_flags_grep_gate() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let allowlist = [
            "src/terrain/bevy_terrain.rs",
            "src/terrain/editor/map_snapshot.rs",
            "src/gui/editor/map_editor/mod.rs",
            "src/dev/proof_grade.rs",
        ];
        let needles = ["features.road", "features.track", "TerrainFeatures {"];
        let src = root.join("src");
        for entry in walkdir_allowlist_only(&src, &allowlist) {
            let rel = entry
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if allowlist.contains(&rel.as_str()) {
                continue;
            }
            let content = std::fs::read_to_string(&entry).unwrap_or_else(|e| panic!("read {rel}: {e}"));
            for needle in needles {
                assert!(
                    !content.contains(needle),
                    "INFRA-E0-002: {rel} must not reference `{needle}` — use transport graph"
                );
            }
        }
    }

    fn walkdir_allowlist_only(src: &PathBuf, allowlist: &[&str]) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let mut stack = vec![src.clone()];
        while let Some(dir) = stack.pop() {
            let Ok(read) = std::fs::read_dir(&dir) else {
                continue;
            };
            for ent in read.flatten() {
                let path = ent.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    out.push(path);
                }
            }
        }
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        out.retain(|p| {
            let rel = p
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            !allowlist.contains(&rel.as_str())
        });
        out
    }

    /// **DEHACK-ENV-002** — `RUST_ENGINE_S7P_STEWARD` sunset; no runtime env readers in activation spine.
    #[test]
    fn dehack_env_002_s7p_steward_sunset() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("src/economy/activation/concrete_chain_e2e.rs");
        let content = std::fs::read_to_string(&path).expect("read concrete_chain_e2e.rs");
        assert!(
            !content.contains(r#"env_on("RUST_ENGINE_S7P_STEWARD")"#)
                && !content.contains(r#"var("RUST_ENGINE_S7P_STEWARD")"#),
            "DEHACK-ENV-002: remove RUST_ENGINE_S7P_STEWARD env reader — use PlayScenarioPlugin"
        );
    }
}
