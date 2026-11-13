//! **CON-P2-002** — sim-only staged site phase advance (no render writes).

use bevy::prelude::*;

use crate::construction::site_stage::SiteStageProgress;
use crate::construction::site_stage_transitions::{
    next_site_stage, phase_progress_step, saturating_phase_progress,
};
use crate::strategic::{
    ConstructionSite,
    PlannedSite, SiteConstructionBook, SiteConstructionPhase, SiteId,
    SiteMaterialStarved, SiteResourceManifest,
};
use crate::systems::sim_control::{sim_step_due, SimControlSystemSet, SimStepClock};

/// Legacy backfill only — **CON-P2-001** attaches on commit; this covers pre-P2 spawns missing progress.
pub fn init_site_stage_progress_for_planned_sites(
    mut commands: Commands,
    q: Query<
        Entity,
        (
            With<ConstructionSite>,
            With<PlannedSite>,
            Without<SiteStageProgress>,
        ),
    >,
) {
    for entity in &q {
        commands.entity(entity).insert(SiteStageProgress::default());
    }
}

/// Advance `SiteStageProgress` and sync `ConstructionSite` + book: one phase step per
/// [`SimStepClock`] step (one simulated second), `due` steps per frame.
///
/// `UnderConstruction` is material-gated: progress never runs ahead of the manifest's delivered
/// ratio (`SiteResourceManifest::delivered_ratio`, filled by logistics), so a starved site stalls
/// and carries [`SiteMaterialStarved`] until the missing material arrives. Sites without a bill
/// (ratio 1) build on the clock alone.
pub fn advance_site_construction_tick_system(
    mut commands: Commands,
    clock: Res<SimStepClock>,
    mut q: Query<(
        Entity,
        &mut ConstructionSite,
        &PlannedSite,
        &mut SiteStageProgress,
        Option<&mut SiteResourceManifest>,
        Has<SiteMaterialStarved>,
    )>,
    mut book: ResMut<SiteConstructionBook>,
) {
    for (entity, mut site, planned, mut stage, mut manifest, was_starved) in &mut q {
        // Steps run per site in order; a site that changes phase keeps stepping in its new phase.
        let mut held_last: Option<SiteMaterialStarved> = None;
        for _ in 0..clock.due {
            held_last = advance_site_one_step(&mut site, planned, &mut stage, manifest.as_deref_mut(), &mut book);
        }
        match (held_last, was_starved) {
            (Some(starved), _) => {
                commands.entity(entity).insert(starved);
            }
            (None, true) => {
                commands.entity(entity).remove::<SiteMaterialStarved>();
            }
            (None, false) => {}
        }
    }
}

/// One step of one site; returns the starvation hold when material capped this step's progress.
fn advance_site_one_step(
    site: &mut ConstructionSite,
    planned: &PlannedSite,
    stage: &mut SiteStageProgress,
    manifest: Option<&mut SiteResourceManifest>,
    book: &mut SiteConstructionBook,
) -> Option<SiteMaterialStarved> {
    if matches!(
        site.phase,
        SiteConstructionPhase::Operational
            | SiteConstructionPhase::Provisioning
            | SiteConstructionPhase::Damaged
            | SiteConstructionPhase::Offline
            | SiteConstructionPhase::Abandoned
    ) {
        return None;
    }

    let dt = phase_progress_step(site.phase);
    if dt <= 0.0 {
        return None;
    }
    let wanted = saturating_phase_progress(stage.progress, dt);
    let mut held_by: Option<SiteMaterialStarved> = None;
    stage.progress = if site.phase == SiteConstructionPhase::UnderConstruction {
        match manifest {
            Some(m) => {
                // Never regress, never run ahead of the scarcest delivered material.
                let ratio = m.min_delivered_ratio();
                let capped = wanted.min(ratio).max(stage.progress);
                if capped < wanted {
                    held_by = Some(SiteMaterialStarved {
                        delivered_ratio: ratio,
                        missing: m.short_materials(),
                    });
                }
                // Built-in material leaves the site's stock as the stage advances.
                m.consume_to_progress(capped);
                capped
            }
            None => wanted,
        }
    } else {
        wanted
    };
    if stage.progress < 1.0 {
        let sid = SiteId(site.site_id);
        if let Some(st) = book.by_site.get_mut(&sid) {
            st.progress = stage.progress;
            st.phase = site.phase;
        }
        return held_by;
    }

    let Some((next_phase, next_sub)) = next_site_stage(site.phase, stage.substep) else {
        return held_by;
    };

    site.phase = next_phase;
    stage.progress = 0.0;
    stage.substep = next_sub;
    site.operational_readiness = if next_phase == SiteConstructionPhase::Operational { 1.0 } else { 0.0 };

    let sid = planned.site_id;
    if let Some(st) = book.by_site.get_mut(&sid) {
        st.phase = next_phase;
        st.progress = 0.0;
    }
    held_by
}





pub struct SiteStageTickPlugin;

impl Plugin for SiteStageTickPlugin {
    fn build(&self, app: &mut App) {
        crate::systems::sim_control::ensure_sim_step_clock(app);
        app.add_systems(
            Update,
            (
                init_site_stage_progress_for_planned_sites,
                ApplyDeferred,
                advance_site_construction_tick_system.run_if(sim_step_due),
            )
                .chain()
                .after(SimControlSystemSet::AdvanceSimTick),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::site_stage::ClearingSubstep;
    use crate::strategic::{
        BuildSiteTile, CommitConstructionSiteEvent, FootprintTiles, LayerType, SiteArchetype,
        SiteConstructionBook, SiteConstructionPhase, SiteIdIssuer,
    };
    use crate::systems::sim_control::{SimTimeMicros, SIM_STEP_MICROS};

    fn assemble_stage_tick_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SiteConstructionBook>()
            .init_resource::<SiteIdIssuer>()
            .add_message::<CommitConstructionSiteEvent>()
            .add_plugins(SiteStageTickPlugin)
            .add_systems(Update, crate::strategic::commit_construction_site_system);
        app
    }

    #[test]
    fn commit_leaves_site_planned_not_operational() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SiteConstructionBook>()
            .init_resource::<SiteIdIssuer>()
            .add_message::<CommitConstructionSiteEvent>()
            .add_systems(Update, crate::strategic::commit_construction_site_system);
        let owner = app.world_mut().spawn_empty().id();
        app.world_mut().write_message(CommitConstructionSiteEvent {
            site_id: crate::strategic::SiteId::UNASSIGNED,
            owner,
            archetype: SiteArchetype::Factory,
            origin: BuildSiteTile { x: 4, z: 4 },
            footprint: FootprintTiles { width: 2, depth: 2 },
            layer: LayerType::Surface,
            catalog_id: None,
            placement: None,
        });
        app.update();
        let world = app.world_mut();
        let site = world
            .query::<&ConstructionSite>()
            .single(world)
            .expect("site");
        assert_eq!(site.phase, SiteConstructionPhase::Planned);
        assert!(site.operational_readiness < 0.01);
    }



    #[test]
    fn forest_pipeline_reaches_operational_in_n_ticks() {
        let mut app = assemble_stage_tick_test_app();
        let owner = app.world_mut().spawn_empty().id();
        app.world_mut().write_message(CommitConstructionSiteEvent {
            site_id: crate::strategic::SiteId::UNASSIGNED,
            owner,
            archetype: SiteArchetype::Factory,
            origin: BuildSiteTile { x: 8, z: 8 },
            footprint: FootprintTiles { width: 2, depth: 2 },
            layer: LayerType::Surface,
            catalog_id: None,
            placement: None,
        });
        app.update();

        let mut saw_clearing_trees = false;
        let mut saw_clearing_stumps = false;
        let max_frames = 16;
        for _ in 0..max_frames {
            app.world_mut().resource_mut::<SimTimeMicros>().0 += SIM_STEP_MICROS;
            app.update();
            let world = app.world_mut();
            let site = world
                .query::<&ConstructionSite>()
                .single(world)
                .expect("site");
            if site.phase == SiteConstructionPhase::Operational {
                assert!(site.operational_readiness >= 1.0);
                return;
            }
            if site.phase == SiteConstructionPhase::Clearing {
                let stage = world
                    .query::<&SiteStageProgress>()
                    .single(world)
                    .expect("stage");
                match stage.substep {
                    Some(ClearingSubstep::Trees) => saw_clearing_trees = true,
                    Some(ClearingSubstep::Stumps) => saw_clearing_stumps = true,
                    None => {}
                }
            }
        }
        let world = app.world_mut();
        let phase = world
            .query::<&ConstructionSite>()
            .single(world)
            .map(|s| s.phase)
            .expect("site");
        let stage_snap = world
            .query::<&SiteStageProgress>()
            .single(world)
            .ok()
            .map(|s| (s.progress, s.substep));
        let (progress, substep) = stage_snap.unwrap_or((0.0, None));
        panic!(
            "forest pipeline did not reach Operational in {max_frames} ticks; phase={phase:?} progress={progress:?} substep={substep:?}; trees={saw_clearing_trees} stumps={saw_clearing_stumps}",
        );
    }
}
