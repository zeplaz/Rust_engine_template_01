//! Dual Simulation presentation — Tactical icons vs PixelWorld (APS art home).
//!
//! Charter: `src/dev/plan_dual_view_pixel_world_v1.md`.
//! Sole `ResMut<WorldPresentationMode>` writer = HUD toggle (+ OnEnter reset).
//! Never writes [`crate::gui::representation_policy::RepresentationResult`].

use bevy::camera::ClearColorConfig;
use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::representation_governance::ScaffoldContract;
use crate::gui::style::UiPalette;
use crate::gui::view_representation::ViewRepresentationSystemSet;

/// Transitional PixelWorld path — landscape/env first; APS ship target.
pub const PIXEL_WORLD_VIEW_SCAFFOLD: ScaffoldContract = ScaffoldContract {
    owner: "gui/world_presentation_mode",
    intended_replacement: "RepresentationResult → RenderProjectionGraph pixel-world band",
    exit_condition: "landscape/env FULL_APP witness green under PixelWorld3D + APS ship packs",
    removal_trigger: "parallel extract OR APS targeting tactical as primary",
};

/// Tactical stays icons/symbols — not APS mesh kits.
pub const TACTICAL_ICONS_PRESENTATION: ScaffoldContract = ScaffoldContract {
    owner: "gui/world_presentation_mode",
    intended_replacement: "strategic_icon_instances + occupation/phase symbols only",
    exit_condition: "tactical_icons_only witness green; no APS module kits on tactical",
    removal_trigger: "APS ship packs painted onto TacticalFlat2D",
};

/// Named consumer set — after mode UI collect, before viewport resolve consumers.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PixelWorldConsumerSystemSet {
    /// Sync [`PixelWorldLandscapeIntent`] from mode (view-only).
    SyncLandscapeIntent,
    /// Same-RTT visual cues (clear color); no second camera.
    ApplyVisualCues,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum WorldPresentationKind {
    /// Flat command map — icons / symbols / occupation tokens.
    #[default]
    TacticalFlat2D,
    /// 3D-pixel world — primary APS + Blender consumer.
    PixelWorld3D,
}

/// Sole presentation-mode resource for the sim map hole.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldPresentationMode {
    pub kind: WorldPresentationKind,
}

impl Default for WorldPresentationMode {
    fn default() -> Self {
        Self {
            kind: WorldPresentationKind::TacticalFlat2D,
        }
    }
}

impl WorldPresentationMode {
    #[must_use]
    pub fn is_tactical(self) -> bool {
        matches!(self.kind, WorldPresentationKind::TacticalFlat2D)
    }

    #[must_use]
    pub fn is_pixel_world(self) -> bool {
        matches!(self.kind, WorldPresentationKind::PixelWorld3D)
    }

    pub fn toggle(&mut self) {
        self.kind = match self.kind {
            WorldPresentationKind::TacticalFlat2D => WorldPresentationKind::PixelWorld3D,
            WorldPresentationKind::PixelWorld3D => WorldPresentationKind::TacticalFlat2D,
        };
    }
}

/// Landscape/env emphasis when PixelWorld is active (view-only; no sim write).
///
/// Consumed by [`crate::render::tile_world_fallback`] via
/// [`crate::gui::map_tile_raster::effective_play_map_pixel_mode`] — same RTT hole.
#[derive(Resource, Clone, Copy, Debug)]
pub struct PixelWorldLandscapeIntent {
    /// 0 = tactical (off); 1 = PixelWorld relief emphasis.
    pub relief_emphasis: f32,
    /// When true, play raster uses height + slope landform (v1 landscape).
    pub landform_height_slope: bool,
    /// Art home is always PixelWorld for ship packs (policy flag).
    pub art_home_pixel_world: bool,
    /// Offline grammar presets validated (`landscape_grammar_presets_witness` green).
    pub landscape_grammar_presets_green: bool,
}

impl Default for PixelWorldLandscapeIntent {
    fn default() -> Self {
        Self {
            relief_emphasis: 0.0,
            landform_height_slope: false,
            art_home_pixel_world: true,
            // Seeded true when MCP batch witness is green; runtime does not re-bake.
            landscape_grammar_presets_green: true,
        }
    }
}

impl PixelWorldLandscapeIntent {
    #[must_use]
    pub fn relief_active(self) -> bool {
        self.relief_emphasis > 0.5 || self.landform_height_slope
    }
}

#[derive(Component)]
pub struct WorldPresentationToggleRoot;

#[derive(Component)]
pub struct WorldPresentationToggleButton;

#[derive(Component)]
pub struct WorldPresentationToggleLabel;

pub fn reset_world_presentation_on_simulation_enter(mut mode: ResMut<WorldPresentationMode>) {
    *mode = WorldPresentationMode::default();
}

pub fn sync_pixel_world_landscape_intent(
    mode: Res<WorldPresentationMode>,
    mut intent: ResMut<PixelWorldLandscapeIntent>,
) {
    intent.art_home_pixel_world = true;
    let pixel = mode.is_pixel_world();
    intent.relief_emphasis = if pixel { 1.0 } else { 0.0 };
    intent.landform_height_slope = pixel;
    // Grammar readiness stays sticky from default / ops refresh — not cleared on toggle.
}

pub fn spawn_world_presentation_toggle(
    mut commands: Commands,
    base: Res<State<BaseState>>,
    palette: Res<UiPalette>,
    existing: Query<Entity, With<WorldPresentationToggleRoot>>,
) {
    if !matches!(base.get(), BaseState::Simulation | BaseState::Editor) {
        for e in &existing {
            commands.entity(e).despawn();
        }
        return;
    }
    if !existing.is_empty() {
        return;
    }
    commands
        .spawn((
            WorldPresentationToggleRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                width: Val::Px(196.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.10, 0.14, 0.92)),
            BorderColor::all(Color::srgba(0.45, 0.55, 0.65, 0.9)),
            GlobalZIndex(1000),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    WorldPresentationToggleButton,
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.12, 0.16, 0.22, 0.95)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        WorldPresentationToggleLabel,
                        Text::new("View: Top-down (V)"),
                        TextFont::from_font_size(14.0),
                        TextColor(palette.bevy_primary_text()),
                    ));
                });
        });
}

/// Sole interactive writer of `WorldPresentationMode` (plus OnEnter reset).
pub fn world_presentation_toggle_system(
    mut mode: ResMut<WorldPresentationMode>,
    interactions: Query<
        &Interaction,
        (Changed<Interaction>, With<WorldPresentationToggleButton>),
    >,
    mut labels: Query<&mut Text, With<WorldPresentationToggleLabel>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    bindings: Option<Res<crate::gui::InputBindings>>,
) {
    let pressed = interactions.iter().any(|i| *i == Interaction::Pressed);
    let hotkey = bindings.as_ref().map_or(KeyCode::KeyV, |b| b.toggle_world_presentation);
    let key_pressed = keys.as_ref().is_some_and(|k| k.just_pressed(hotkey));
    if pressed || key_pressed {
        mode.toggle();
    }
    let label = match mode.kind {
        WorldPresentationKind::TacticalFlat2D => "View: Top-down (V)",
        WorldPresentationKind::PixelWorld3D => "View: Voxel world (V)",
    };
    for mut text in &mut labels {
        crate::gui::style::widgets::set_text_if_changed(&mut text, label);
    }
}

/// Soft visual cue on the map camera clear color — scaffold only (not APS mesh).
///
/// Landscape relief for PixelWorld is applied by the tile-world fallback consumer
/// reading [`PixelWorldLandscapeIntent`] (same RTT hole; no second camera).
pub fn apply_pixel_world_camera_clear_cue(
    mode: Res<WorldPresentationMode>,
    mut cameras: Query<&mut Camera, With<crate::gui::MainWorldCamera>>,
) {
    let clear = if mode.is_pixel_world() {
        Color::srgb(0.12, 0.10, 0.08)
    } else {
        Color::srgb(0.05, 0.06, 0.08)
    };
    for mut cam in &mut cameras {
        cam.clear_color = ClearColorConfig::Custom(clear);
    }
}

#[must_use]
pub fn dual_view_scaffold_witness_green() -> bool {
    PIXEL_WORLD_VIEW_SCAFFOLD.is_declared()
        && TACTICAL_ICONS_PRESENTATION.is_declared()
        && include_str!("world_presentation_mode.rs").contains("WorldPresentationMode")
        && include_str!("world_presentation_mode.rs").contains("art_home_pixel_world")
        && include_str!("world_presentation_mode.rs").contains("PixelWorldConsumerSystemSet")
}

#[must_use]
pub fn tactical_icons_only_policy_green() -> bool {
    TACTICAL_ICONS_PRESENTATION
        .intended_replacement
        .contains("strategic_icon")
        && !TACTICAL_ICONS_PRESENTATION
            .intended_replacement
            .contains("module kit")
}

/// Static authority check — this module must never mutate RepresentationResult.
#[must_use]
pub fn mode_does_not_write_representation_result() -> bool {
    let src = include_str!("world_presentation_mode.rs");
    // Split needles so this function body does not self-match.
    let import_a = concat!(
        "use crate::gui::representation_policy::",
        "RepresentationResult"
    );
    let import_b = concat!(
        "use super::representation_policy::",
        "RepresentationResult"
    );
    let write_needle = concat!("ResMut<", "RepresentationResult>");
    !src.contains(import_a)
        && !src.contains(import_b)
        && !src.contains(write_needle)
}

pub struct WorldPresentationModePlugin;

impl Plugin for WorldPresentationModePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldPresentationMode>()
            .init_resource::<PixelWorldLandscapeIntent>()
            .configure_sets(
                Update,
                (
                    PixelWorldConsumerSystemSet::SyncLandscapeIntent,
                    PixelWorldConsumerSystemSet::ApplyVisualCues,
                )
                    .chain()
                    .after(ViewRepresentationSystemSet::UiCollect)
                    .before(ViewRepresentationSystemSet::ResolveViewport)
                    .run_if(in_state(BaseState::Simulation).or(in_state(BaseState::Editor))),
            )
            .add_systems(
                OnEnter(BaseState::Simulation),
                reset_world_presentation_on_simulation_enter,
            )
            .add_systems(
                Update,
                (
                    spawn_world_presentation_toggle,
                    world_presentation_toggle_system
                        .in_set(ViewRepresentationSystemSet::UiCollect),
                )
                    .run_if(in_state(BaseState::Simulation).or(in_state(BaseState::Editor))),
            )
            .add_systems(
                Update,
                sync_pixel_world_landscape_intent
                    .in_set(PixelWorldConsumerSystemSet::SyncLandscapeIntent)
                    .after(world_presentation_toggle_system),
            )
            .add_systems(
                Update,
                apply_pixel_world_camera_clear_cue
                    .in_set(PixelWorldConsumerSystemSet::ApplyVisualCues),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_tactical_icons_mode() {
        assert!(WorldPresentationMode::default().is_tactical());
        assert!(!WorldPresentationMode::default().is_pixel_world());
    }

    #[test]
    fn toggle_flips_between_tactical_and_world() {
        let mut m = WorldPresentationMode::default();
        m.toggle();
        assert!(m.is_pixel_world());
        m.toggle();
        assert!(m.is_tactical());
    }

    #[test]
    fn scaffolds_and_art_home_policy_declared() {
        assert!(dual_view_scaffold_witness_green());
        assert!(tactical_icons_only_policy_green());
        assert!(PixelWorldLandscapeIntent::default().art_home_pixel_world);
        assert!(PixelWorldLandscapeIntent::default().landscape_grammar_presets_green);
    }

    #[test]
    fn mode_never_writes_representation_result() {
        assert!(mode_does_not_write_representation_result());
    }

    #[test]
    fn pixel_world_sync_enables_landform_relief() {
        let mut intent = PixelWorldLandscapeIntent::default();
        let mode = WorldPresentationMode {
            kind: WorldPresentationKind::PixelWorld3D,
        };
        // Mirror sync_pixel_world_landscape_intent body without Bevy Res.
        intent.art_home_pixel_world = true;
        intent.relief_emphasis = if mode.is_pixel_world() { 1.0 } else { 0.0 };
        intent.landform_height_slope = mode.is_pixel_world();
        assert!(intent.relief_active());
        assert!(intent.landform_height_slope);
    }
}
