//! Compile-time guard: fail fast on banned import roots (see `prompts/matrix/repo/repo_boundary_matrix_v1.md`).
//! Skips known legacy files that are intentionally kept off the active module graph.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const BANNED: &[&str] = &[
    // Wrong crate root paths (pre-module-graph imports)
    "crate::production::",
    "crate::resource::",
    "crate::entity::",
    "crate::entities::e_componets::",
    "crate::entities::damages::",
    "crate::entities::e_states::",
    "crate::entities::e_flag_types::",
    "crate::entities::strukturave::",
    "crate::entities::production::power_comps::",
    "crate::io::deserialzers::",
    // Old Bevy UI bundle types (removed in 0.15 → 0.18 path)
    "NodeBundle",
    "TextBundle",
    "ButtonBundle",
    "ImageBundle",
    // Old Bevy color API (renamed to srgb in 0.14)
    "Color::rgb(",
    "Color::rgba(",
    // Old egui context type (replaced by EguiContexts)
    "ResMut<EguiContext>",
    // Old input type (renamed to ButtonInput in 0.14)
    "Res<Input<",
];

const LEGACY_EXACT_FILES: &[&str] = &[
    "entities/production/concrete/sys.rs",
    "entities/production/aluminum/production_sys.rs",
    "entities/production/prod_comps.rs",
    "systems/production/production_consumption.rs",
    "engine/engine.rs",
    "render/light.rs",
    "gui/in_game_ui.rs",
    // Binaries and glue stubs that may contain intentional old names in comments
    "bin/world_generator.rs",
];

fn is_legacy_path(path: &Path) -> bool {
    let s = path.to_string_lossy();
    LEGACY_EXACT_FILES
        .iter()
        .any(|suffix| s.replace('\\', "/").ends_with(suffix))
}

fn scan_dir(dir: &Path, offenders: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, offenders);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if is_legacy_path(&path) {
                continue;
            }
            let Ok(contents) = fs::read_to_string(&path) else {
                continue;
            };
            let first_line = contents.lines().next().unwrap_or("");
            if first_line.contains("LEGACY MODULE (not actively wired)") {
                continue;
            }
            for ban in BANNED {
                if contents.contains(ban) {
                    offenders.push(format!("{} contains banned `{ban}`", path.display()));
                }
            }
        }
    }
}

fn main() {
    if env::var_os("CARGO_FEATURE_RESEARCH_LMODELS").is_some() {
        println!("cargo:warning=feature `research_lmodels`: `engine::lmodels` is a stub — linfa crates are optional deps only and are not wired; replace the stub when implementing ML integration.");
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let src = PathBuf::from(manifest_dir).join("src");
    let mut offenders = Vec::new();
    scan_dir(&src, &mut offenders);
    if !offenders.is_empty() {
        eprintln!("banned-import check failed:");
        for o in &offenders {
            eprintln!("  {o}");
        }
        panic!("fix banned imports or mark file as LEGACY MODULE (see build.rs)");
    }
}
