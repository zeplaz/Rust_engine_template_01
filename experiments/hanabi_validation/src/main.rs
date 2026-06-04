//! Write `report_v1.md` from measured spike presets (no main-engine side effects).

use std::path::PathBuf;

use hanabi_validation::build_default_report;

fn main() {
    let report = build_default_report();
    let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"))
        .join("report_v1.md");
    std::fs::write(&path, &report).unwrap_or_else(|e| {
        panic!("failed to write {}: {e}", path.display());
    });
    println!("wrote {}", path.display());
}
