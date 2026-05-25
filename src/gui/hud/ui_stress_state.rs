//! Presentation-only stress mirror for minimap chrome (UX-E01 M1 stub).

use bevy::prelude::*;

use crate::render::{EcologyVisualSnapshot, SharedOverlayFieldBuffers};

/// Read-only HUD stress aggregates — never writes sim or overlay buffers.
#[derive(Resource, Clone, Debug, Default)]
pub struct UiStressState {
    pub fire_pressure: f32,
    pub ecology_stress: f32,
    pub revision: u64,
    /// Witness: presentation lane must not mutate sim authority.
    pub ui_stress_wrote_sim: bool,
}

pub fn sync_ui_stress_from_sim_system(
    overlay: Option<Res<SharedOverlayFieldBuffers>>,
    ecology: Option<Res<EcologyVisualSnapshot>>,
    mut stress: ResMut<UiStressState>,
) {
    stress.ui_stress_wrote_sim = false;
    let peak_fire = overlay
        .as_ref()
        .map(|o| {
            o.chunk_fire_heat
                .values()
                .copied()
                .fold(0.0_f32, f32::max)
        })
        .unwrap_or(0.0);
    stress.fire_pressure = peak_fire.clamp(0.0, 1.0);
    stress.ecology_stress = ecology
        .as_ref()
        .map(|e| e.mean_fire_risk.clamp(0.0, 1.0))
        .unwrap_or(0.0);
    stress.revision = stress.revision.wrapping_add(1);
}

#[must_use]
pub fn minimap_stress_chrome_enabled() -> bool {
    std::env::var("MINIMAP_STRESS_CHROME")
        .ok()
        .is_some_and(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

pub fn apply_minimap_stress_chrome_system(
    stress: Res<UiStressState>,
    palette: Res<crate::gui::style::UiPalette>,
    mut chrome: Query<&mut BorderColor, With<super::simulation_shell_phase2::MinimapChromeRoot>>,
) {
    if !minimap_stress_chrome_enabled() {
        return;
    }
    let warm = palette.bevy_accent_terminal();
    let cool = palette.bevy_accent_terminal();
    let mix = (stress.fire_pressure * 0.7 + stress.ecology_stress * 0.3).clamp(0.0, 1.0);
    let tint = warm.mix(&cool, 1.0 - mix);
    for mut border in &mut chrome {
        *border = BorderColor::all(tint.with_alpha(0.85));
    }
}
