//! **CON-P2-002** — sim-only staged site phase advance (no render writes).

use bevy::prelude::*;

use crate::construction::site_stage::SiteStageProgress;
use crate::construction::site_stage_transitions::next_site_stage;
use crate::strategic::{
    BuildSiteTile, CommitConstructionSiteEvent, ConstructionSite, FootprintTiles, LayerType,
    PlannedSite, SiteArchetype, SiteConstructionBook, SiteConstructionPhase, SiteId,
    SiteIdIssuer,
};
use crate::systems::sim_control::{SimControlState, SimControlSystemSet, SimTick};

/// v1 stub: one phase-unit per second at speed 1.0 (tunable per archetype later).
const DEFAULT_PHASE_RATE: f32 = 1.0;

/// Legacy backfill only — **CON-P2-001** attaches on commit; this covers pre-P2 spawns missing progress.
pub fn init_site_stage_progress_for_planned_sites(
    mut commands: Commands,
    q: Query<Entity, (With<ConstructionSite>, With<PlannedSite>, Without<SiteStageProgress>)>,
) {
    for entity in &q {
        commands.entity(entity).insert(SiteStageProgress::default());
    }
}

/// Advance `SiteStageProgress` and sync `ConstructionSite` + book (one phase step per sim tick).
pub fn advance_site_construction_tick_system(
    tick: Res<SimTick>,
    ctrl: Res<SimControlState>,
    mut last_tick: Local<Option<u64>>,
    mut q: Query<(
        &mut ConstructionSite,
        &PlannedSite,
        &mut SiteStageProgress,
    )>,
    mut book: ResMut<SiteConstructionBook>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let current = tick.0;
    if last_tick.is_some_and(|prev| prev == current) {
        return;
    }
    *last_tick = Some(current);
    let dt = DEFAULT_PHASE_RATE;

    for (mut site, planned, mut stage) in &mut q {
        if matches!(
            site.phase,
            SiteConstructionPhase::Operational
                | SiteConstructionPhase::Damaged
                | SiteConstructionPhase::Offline
                | SiteConstructionPhase::Abandoned
        ) {
            continue;
        }

        stage.progress = (stage.progress + dt).min(1.0);
        if stage.progress < 1.0 {
            let sid = SiteId(site.site_id);
            if let Some(st) = book.by_site.get_mut(&sid) {
                st.progress = stage.progress;
                st.phase = site.phase;
            }
            continue;
        }

        let Some((next_phase, next_sub)) = next_site_stage(site.phase, stage.substep) else {
            continue;
        };

        site.phase = next_phase;
        stage.progress = 0.0;
        stage.substep = next_sub;

        match next_phase {
            SiteConstructionPhase::Operational => {
                site.operational_readiness = 1.0;
            }
            SiteConstructionPhase::UnderConstruction => {
                site.operational_readiness = 0.0;
            }
            _ => {
                site.operational_readiness = 0.0;
            }
        }

        let sid = planned.site_id;
        if let Some(st) = book.by_site.get_mut(&sid) {
            st.phase = next_phase;
            st.progress = 0.0;
        }
    }
}

/// **CON-P2-002** witness rollup (post **CON-P2-001**): commit attaches progress; tick advances phases.
#[must_use]
pub fn construction_site_stage_tick_002_witness_green() -> bool {
    construction_site_stage_tick_002_self_check().is_ok()
}

fn assemble_stage_tick_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SiteConstructionBook>()
        .init_resource::<SiteIdIssuer>()
        .init_resource::<SimControlState>()
        .init_resource::<SimTick>()
        .add_message::<CommitConstructionSiteEvent>()
        .add_plugins(SiteStageTickPlugin)
        .add_systems(Update, crate::strategic::commit_construction_site_system);
    app
}

fn construction_site_stage_tick_002_self_check() -> Result<(), &'static str> {
    let mut app = assemble_stage_tick_test_app();
    let owner = app.world_mut().spawn_empty().id();
    app.world_mut().write_message(CommitConstructionSiteEvent {
        site_id: crate::strategic::SiteId::UNASSIGNED,
        owner,
        archetype: SiteArchetype::Factory,
        origin: BuildSiteTile { x: 6, z: 6 },
        footprint: FootprintTiles {
            width: 2,
            depth: 2,
        },
        layer: LayerType::Surface,
        catalog_id: None,
        placement: None,
    });
    app.update();

    {
        let world = app.world_mut();
        let stage_count = world
            .query::<&SiteStageProgress>()
            .iter(world)
            .count();
        if stage_count != 1 {
            return Err("CON-P2-001: SiteStageProgress must attach on commit");
        }
        let site = world
            .query::<&ConstructionSite>()
            .single(world)
            .map_err(|_| "missing ConstructionSite")?;
        if site.phase != SiteConstructionPhase::Planned {
            return Err("commit must leave Planned");
        }
    }

    app.add_plugins(SiteStageTickPlugin);
    app.world_mut().resource_mut::<SimTick>().0 += 1;
    app.update();

    let world = app.world_mut();
    let phase = world
        .query::<&ConstructionSite>()
        .single(world)
        .map_err(|_| "missing ConstructionSite after tick")?
        .phase;
    let progress = world
        .query::<&SiteStageProgress>()
        .single(world)
        .map_err(|_| "missing SiteStageProgress after tick")?
        .progress;
    if phase == SiteConstructionPhase::Planned && progress <= 0.0 {
        return Err("tick must advance stage progress or phase");
    }
    Ok(())
}

pub struct SiteStageTickPlugin;

impl Plugin for SiteStageTickPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                init_site_stage_progress_for_planned_sites,
                ApplyDeferred,
                advance_site_construction_tick_system,
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
    use crate::systems::sim_control::SimTick;

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
            footprint: FootprintTiles {
                width: 2,
                depth: 2,
            },
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
    fn con_p2_002_post_a001_commit_attached_before_tick() {
        assert!(construction_site_stage_tick_002_witness_green());
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
            footprint: FootprintTiles {
                width: 2,
                depth: 2,
            },
            layer: LayerType::Surface,
            catalog_id: None,
            placement: None,
        });
        app.update();

        let mut saw_clearing_trees = false;
        let mut saw_clearing_stumps = false;
        let max_frames = 16;
        for _ in 0..max_frames {
            app.world_mut().resource_mut::<SimTick>().0 += 1;
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
