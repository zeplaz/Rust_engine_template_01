//! Cached save manifest for streaming — one disk read per bundle dir per session.

use std::path::{Path, PathBuf};

use bevy::prelude::*;

use crate::io::save::SaveWorldManifest;

use super::hydrate::load_manifest_for_streaming;

#[derive(Resource, Debug, Default)]
pub struct StreamingManifestCache {
    bundle_dir: PathBuf,
    manifest: Option<SaveWorldManifest>,
}

impl StreamingManifestCache {
    /// Returns cached manifest, reloading when `bundle_dir` changes.
    #[must_use]
    pub fn manifest_for_bundle(&mut self, bundle_dir: &Path) -> Option<&SaveWorldManifest> {
        if self.bundle_dir.as_path() != bundle_dir {
            self.bundle_dir = bundle_dir.to_path_buf();
            self.manifest = load_manifest_for_streaming(bundle_dir);
        } else if self.manifest.is_none() {
            self.manifest = load_manifest_for_streaming(bundle_dir);
        }
        self.manifest.as_ref()
    }

    pub fn invalidate(&mut self) {
        self.manifest = None;
    }
}
