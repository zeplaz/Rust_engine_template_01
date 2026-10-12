//! Left column: mobility profile + terrain tag pool (generation / highlight).


use crate::gui::editor::world_gen_hints as hints;
use crate::gui::editor::world_gen_ui::WorldGenUiState;
use crate::gui::style::{
    muted_label, section_heading, v_space, CmdHeadingStyle, UiPalette, UiSpacing, VertSpace,
    widget_scroll_vertical_capped,
};
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::material::TagId;
use bevy::prelude::*;

#[inline]
fn tt() {}

#[allow(dead_code)] // Tag pool and mobility controls live on the Bevy WorldGen shell.
pub fn world_preview_sidebar() {}
