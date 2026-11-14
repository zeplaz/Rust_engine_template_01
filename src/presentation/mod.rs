//! Presentation contracts (sprite ASM, visual phases) — never write sim truth.

pub mod truck_view;
pub mod visual_asm;

use bevy::prelude::*;

use truck_view::{sync_truck_presentation, TruckPresentationIndex};
use visual_asm::tick_visual_action_age;

use crate::combat::{Formation, Posture};
use crate::entities::production::core::ManufacturingNode;
use crate::systems::fire::{SiteFireStage, SiteFireState};

pub use truck_view::TruckPresentation;
pub use visual_asm::{VisualActionPhase, VisualActionState};

/// Stamp [`VisualActionState`] on formations / plants; sync truck proxies; tick age.
pub struct PresentationAsmPlugin;

impl Plugin for PresentationAsmPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TruckPresentationIndex>().add_systems(
            Update,
            (
                stamp_visual_action_on_formations,
                stamp_visual_action_on_plants,
                stamp_visual_action_on_sites,
                sync_formation_visual_phase,
                sync_plant_visual_from_mfg,
                sync_site_visual_from_fire,
                sync_truck_presentation,
                tick_visual_action_age,
            ),
        );
    }
}

fn stamp_visual_action_on_formations(
    mut commands: Commands,
    q: Query<(Entity, &Formation), (Added<Formation>, Without<VisualActionState>)>,
) {
    for (e, f) in &q {
        let phase = match f.posture {
            Posture::Move | Posture::Attack => VisualActionPhase::Move,
            Posture::Retreat => VisualActionPhase::Damaged,
            Posture::Hold => {
                if f.fortify {
                    VisualActionPhase::Work
                } else {
                    VisualActionPhase::Idle
                }
            }
        };
        commands.entity(e).insert(VisualActionState {
            phase,
            phase_age_secs: 0.0,
            facing: 0,
        });
    }
}

fn stamp_visual_action_on_plants(
    mut commands: Commands,
    q: Query<Entity, (Added<ManufacturingNode>, Without<VisualActionState>)>,
) {
    for e in &q {
        commands.entity(e).insert(VisualActionState::default());
    }
}

fn stamp_visual_action_on_sites(
    mut commands: Commands,
    q: Query<Entity, (Added<SiteFireState>, Without<VisualActionState>)>,
) {
    for e in &q {
        commands.entity(e).insert(VisualActionState::default());
    }
}

fn sync_formation_visual_phase(
    mut q: Query<(&Formation, &mut VisualActionState), Changed<Formation>>,
) {
    for (f, mut vis) in &mut q {
        let phase = match f.posture {
            Posture::Move | Posture::Attack => VisualActionPhase::Move,
            Posture::Retreat => VisualActionPhase::Damaged,
            Posture::Hold => {
                if f.engaged {
                    VisualActionPhase::Alert
                } else if f.fortify {
                    VisualActionPhase::Work
                } else {
                    VisualActionPhase::Idle
                }
            }
        };
        vis.set_phase(phase);
    }
}

fn sync_plant_visual_from_mfg(
    mut q: Query<(&ManufacturingNode, &mut VisualActionState)>,
) {
    for (node, mut vis) in &mut q {
        let phase = if node.current_throughput > 0.05 {
            VisualActionPhase::Work
        } else {
            VisualActionPhase::Idle
        };
        vis.set_phase(phase);
    }
}

/// Presentation only: burn stages drive Damaged / Destroyed ASM (never writes sim).
fn sync_site_visual_from_fire(
    mut q: Query<(&SiteFireState, &mut VisualActionState)>,
) {
    for (fire, mut vis) in &mut q {
        let phase = match fire.stage {
            SiteFireStage::Clear | SiteFireStage::Threatened => continue,
            SiteFireStage::Disrupted => VisualActionPhase::Alert,
            SiteFireStage::Deteriorating => VisualActionPhase::Damaged,
            SiteFireStage::Collapsing => VisualActionPhase::Destroyed,
        };
        vis.set_phase(phase);
    }
}
