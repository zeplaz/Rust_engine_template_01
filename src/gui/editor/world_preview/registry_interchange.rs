//! Canonical terrain registry interchange paths for external desktop asset tooling.

use std::path::{Path, PathBuf};
use std::process::Command;

fn terrain_config_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/config/terrain")
}

fn prefer_ron_then_json(base: &str) -> PathBuf {
    let dir = terrain_config_dir();
    let ron = dir.join(format!("{base}.example.ron"));
    if ron.exists() {
        ron
    } else {
        dir.join(format!("{base}.example.json"))
    }
}

#[must_use]
pub fn material_registry_interchange_path() -> PathBuf {
    prefer_ron_then_json("material_registry")
}

#[must_use]
pub fn tag_registry_interchange_path() -> PathBuf {
    prefer_ron_then_json("tag_registry")
}

/// Reveal the interchange file in the desktop shell (Windows: Explorer select).
pub fn open_registry_interchange_in_desktop_shell(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("registry interchange path missing: {}", path.display()),
        ));
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg("-R").arg(path).spawn()?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(parent) = path.parent() {
            Command::new("xdg-open").arg(parent).spawn()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_interchange_paths_prefer_committed_examples() {
        let material = material_registry_interchange_path();
        let tags = tag_registry_interchange_path();
        assert!(material.exists(), "{}", material.display());
        assert!(tags.exists(), "{}", tags.display());
        assert!(
            material.extension().is_some_and(|ext| ext == "ron" || ext == "json"),
            "{}",
            material.display()
        );
    }
}
