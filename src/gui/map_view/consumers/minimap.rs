//! Minimap egui sampling through the shared map-view backend.

use bevy::prelude::*;

use crate::gui::hud::shell_framework::{shell_widget_runs_egui, HudWidgetId};
use crate::gui::hud::HudDockRegistry;
use crate::gui::map_view::presentation::{MapViewInstanceId, MapViewReadyStates};
use crate::gui::map_view::projection::ResolvedMapViewFrames;
use crate::gui::map_view::texture_cache::MapViewTextureCache;
use crate::gui::{MinimapShellState};
use crate::render::{SimMinimapUiState, TileWorldFallbackState};

pub fn resolve_minimap_egui_texture() {}
