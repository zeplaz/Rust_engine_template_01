//! Layer toggles for volumetric / distortion passes (`base_fire2_smoke.md` §7).
//!
//! **ES-4 ALTERNATIVE:** composites are quarantined — toggles stay off and are **not**
//! driven by camera weights (would falsely claim live passes). See [`super::gpu_paths`].

use bevy::prelude::*;

use super::gpu_paths::{ATMOSPHERE_COMPOSITE_WIRED, ATMOSPHERE_WGSL_QUARANTINED};

#[derive(Resource, Debug, Clone, Copy)]
pub struct AtmosphereRenderLayers {
    pub ground_haze: bool,
    pub smoke_columns: bool,
    pub heat_distortion: bool,
    pub ashfall: bool,
}

impl Default for AtmosphereRenderLayers {
    fn default() -> Self {
        Self {
            ground_haze: false,
            smoke_columns: false,
            heat_distortion: false,
            ashfall: false,
        }
    }
}

impl AtmosphereRenderLayers {
    /// True when any composite toggle is on (must stay false while quarantined).
    #[must_use]
    pub fn any_composite_enabled(&self) -> bool {
        self.ground_haze || self.smoke_columns || self.heat_distortion || self.ashfall
    }
}

/// Pin layers off while WGSL composites remain unwired (ES-4 quarantine).
fn pin_atmosphere_render_layers_quarantined(mut layers: ResMut<AtmosphereRenderLayers>) {
    debug_assert!(!ATMOSPHERE_COMPOSITE_WIRED);
    debug_assert!(ATMOSPHERE_WGSL_QUARANTINED);
    if layers.any_composite_enabled() {
        *layers = AtmosphereRenderLayers::default();
    }
}

pub fn render_layer_systems(app: &mut App) {
    app.init_resource::<AtmosphereRenderLayers>().add_systems(
        Update,
        // Quarantine pin only — camera sync deferred until real pass table (ES-4 wire path).
        pin_atmosphere_render_layers_quarantined
            .in_set(super::pipeline::AtmospherePipelineSet::RenderPrep),
    );
}

#[cfg(test)]
mod tests {
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    use crate::gui::InputBindings;
    use crate::systems::atmosphere::{
        AtmospherePlugin, ATMOSPHERE_COMPOSITE_WIRED, ATMOSPHERE_WGSL_QUARANTINED,
    };
    use crate::systems::sim_control::SimControlPlugin;

    use super::AtmosphereRenderLayers;

    #[test]
    fn render_layers_stay_off_under_quarantine() {
        assert!(!ATMOSPHERE_COMPOSITE_WIRED);
        assert!(ATMOSPHERE_WGSL_QUARANTINED);

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(InputPlugin);
        app.init_resource::<InputBindings>();
        app.add_plugins(SimControlPlugin);
        app.add_plugins(AtmospherePlugin);

        // Try to enable layers (simulating stale writer); pin must clear them.
        app.world_mut().resource_mut::<AtmosphereRenderLayers>().ground_haze = true;
        app.world_mut().resource_mut::<AtmosphereRenderLayers>().ashfall = true;
        app.update();

        let layers = app.world().resource::<AtmosphereRenderLayers>();
        assert!(
            !layers.any_composite_enabled(),
            "ES-4 quarantine must keep AtmosphereRenderLayers all-off"
        );
    }
}
