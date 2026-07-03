//! Map texture source enum for map-view consumers.

use bevy::prelude::*;

/// Authoritative pixel source for a map consumer (resolved by the backend, not egui).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MapTextureSource {
    GpuRenderTarget(Handle<Image>),
    SharedCpuRaster(Handle<Image>),
}

impl MapTextureSource {
    #[must_use]
    pub fn handle(&self) -> &Handle<Image> {
        match self {
            Self::GpuRenderTarget(handle) | Self::SharedCpuRaster(handle) => handle,
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::GpuRenderTarget(_) => "GpuRenderTarget",
            Self::SharedCpuRaster(_) => "SharedCpuRaster",
        }
    }
}

impl Default for MapTextureSource {
    fn default() -> Self {
        Self::SharedCpuRaster(Handle::default())
    }
}
