//! Registry snapshot artifacts â€” save names, not runtime ids.

use std::fs;
use std::io;
use std::path::Path;

use crate::io::save::manifest::RegistrySnapshotRef;

#[must_use]
pub fn build_default_registry_snapshot_refs() -> Vec<RegistrySnapshotRef> {
    vec![
        RegistrySnapshotRef {
            registry_name: "material_registry".into(),
            artifact_path: "registries/material_registry.ron".into(),
        },
        RegistrySnapshotRef {
            registry_name: "tag_registry".into(),
            artifact_path: "registries/tag_registry.ron".into(),
        },
        RegistrySnapshotRef {
            registry_name: "terrain_family_registry".into(),
            artifact_path: "registries/terrain_family_registry.ron".into(),
        },
    ]
}

fn terrain_config_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/config/terrain")
}

fn example_registry_source_path(file_stem: &str) -> std::path::PathBuf {
    let dir = terrain_config_dir();
    let ron = dir.join(format!("{file_stem}.example.ron"));
    let json = dir.join(format!("{file_stem}.example.json"));
    if ron.exists() { ron } else { json }
}

pub fn write_registry_snapshot_artifacts(bundle_dir: &Path) -> io::Result<()> {
    let registries_dir = bundle_dir.join("registries");
    fs::create_dir_all(&registries_dir)?;
    for (stem, artifact_name) in [
        ("material_registry", "material_registry.ron"),
        ("tag_registry", "tag_registry.ron"),
        ("terrain_family_registry", "terrain_family_registry.ron"),
    ] {
        let source = example_registry_source_path(stem);
        if !source.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("missing registry source for {stem}: {}", source.display()),
            ));
        }
        fs::copy(source, registries_dir.join(artifact_name))?;
    }
    Ok(())
}
