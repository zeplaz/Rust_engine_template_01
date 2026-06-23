//! Deep debug Bevy plugin — diagnostics + witness flush.

use bevy::diagnostic::{EntityCountDiagnosticsPlugin, LogDiagnosticsPlugin, SystemInformationDiagnosticsPlugin};
use bevy::prelude::*;

use super::frame_probe::plugin_diagnostics;
use super::latch::{deep_debug_active, init_startup_config};
use super::minimap_trace::MinimapDeepTrace;
use super::witness::{deep_debug_post_update, log_deep_debug_banner};
use super::frame_probe::DeepDebugFrameProbe;

pub struct EngineDeepDebugPlugin;

impl Plugin for EngineDeepDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MinimapDeepTrace>()
            .init_resource::<DeepDebugFrameProbe>()
            .init_resource::<super::subsystem_cache::DeepDebugSubsystemCache>()
            .add_systems(Startup, (init_startup_config, log_deep_debug_banner).chain())
            .add_systems(
                PostUpdate,
                super::subsystem_cache::refresh_deep_debug_subsystem_cache
                    .before(deep_debug_post_update),
            )
            .add_systems(PostUpdate, deep_debug_post_update);
        if deep_debug_active() {
            plugin_diagnostics(app);
            app.add_plugins((
                LogDiagnosticsPlugin::default(),
                EntityCountDiagnosticsPlugin::default(),
                SystemInformationDiagnosticsPlugin,
            ));
        }
        app.add_systems(
            PostUpdate,
            super::minimap_trace::snapshot_minimap_after_compositor_pass
                .after(crate::render::minimap_compositor::run_minimap_compositor_pass),
        );
    }
}
