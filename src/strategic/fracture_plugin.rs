//! **Fracture overlay** — secondary instability lens: soft events + readouts only (no topology / war authority).

use bevy::prelude::*;

use super::behavior_fracture::{
    fracture_event_emit_system, sub_faction_stub_hook_system, FractureEventBus, FractureOverlaySettings,
    FractureProbabilityOverlay, FractureSignalBus, FractureSignalScratch, FractureStageScratch,
};
use super::strategic_behavior_schedule::StrategicBehaviorSchedule;

pub struct FracturePlugin;

impl Plugin for FracturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FractureSignalBus>()
            .init_resource::<FractureEventBus>()
            .init_resource::<FractureSignalScratch>()
            .init_resource::<FractureStageScratch>()
            .init_resource::<FractureOverlaySettings>()
            .init_resource::<FractureProbabilityOverlay>()
            .add_systems(
                Update,
                fracture_event_emit_system.in_set(StrategicBehaviorSchedule::FractureOverlay),
            )
            .add_systems(PostUpdate, sub_faction_stub_hook_system);
    }
}
