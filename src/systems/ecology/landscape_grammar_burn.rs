//! VEG-BURN-OVERLAY-001..003 — `ActiveBurn` transient overlay + burn SM + MACRO-REGROWTH-CHAIN.
//!
//! Plan: `src/dev/plan_veg_burn_extract_001_v1.md`

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::landscape_grammar_lg2::{
    DisturbanceHistory, LandscapeGrammarLg2Witness, SuccessionState, SuccessionTopologyStage,
};
use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};
use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use crate::systems::fire::ChunkSurfaceFire;
use crate::systems::sim_control::SimTick;
use crate::terrain::generation::Chunk;

pub const LANDSCAPE_GRAMMAR_BURN_OVERLAY_LIVE_JSON: &str =
    "debug_runs/landscape_grammar_burn_overlay_live.json";

pub const VEG_BURN_FRAME_COUNT: u8 = 8;
pub const VEG_BURN_FRAME_PERIOD_MS: u32 = 250;
pub const ACTIVE_BURN_HEAT_EPS: f32 = 0.05;
pub const ACTIVE_BURN_IGNITE_HEAT: f32 = 0.35;

/// MACRO-REGROWTH-CHAIN phases (plan §3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegrowthMacroPhase {
    #[default]
    None,
    Scar,
    Nuclei,
    Front,
    Closing,
    Mature,
}

/// Transient burn overlay — not a replacement for [`SuccessionState`] (P-004).
#[derive(Component, Clone, Debug)]
pub struct ActiveBurn {
    pub heat: f32,
    pub frame_index: u8,
    pub started_tick: u64,
    pub severity: f32,
    pub regrowth_macro_phase: RegrowthMacroPhase,
    pub cooled_at_tick: Option<u64>,
}

impl Default for ActiveBurn {
    fn default() -> Self {
        Self {
            heat: 0.0,
            frame_index: 0,
            started_tick: 0,
            severity: 0.0,
            regrowth_macro_phase: RegrowthMacroPhase::None,
            cooled_at_tick: None,
        }
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct LandscapeBurnWitness {
    pub active_burn_chunks: u32,
    pub regrowth_macro_advances: u32,
    pub overlay_removals: u32,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LandscapeBurnSet {
    ApplyActiveBurn,
    AdvanceRegrowthMacro,
    RemoveMatureOverlay,
}

#[must_use]
pub fn veg_burn_frame_index(sim_tick: u64, started_tick: u64) -> u8 {
    let elapsed_ms = sim_tick.saturating_sub(started_tick).saturating_mul(16);
    let period = u64::from(VEG_BURN_FRAME_PERIOD_MS.max(1));
    let frame = (elapsed_ms / period) % u64::from(VEG_BURN_FRAME_COUNT.max(1));
    u8::try_from(frame).unwrap_or(0)
}

#[must_use]
pub fn variant_key_for_burn_row(
    burn: Option<&ActiveBurn>,
    stage: SuccessionTopologyStage,
) -> String {
    if let Some(b) = burn {
        if b.heat > ACTIVE_BURN_HEAT_EPS {
            return format!("veg_burn_{:02}", b.frame_index.min(VEG_BURN_FRAME_COUNT - 1));
        }
        return match b.regrowth_macro_phase {
            RegrowthMacroPhase::Nuclei => "veg_regrowth_nuclei".into(),
            RegrowthMacroPhase::Front => "veg_regrowth_front".into(),
            RegrowthMacroPhase::Scar | RegrowthMacroPhase::Closing => "veg_damaged".into(),
            RegrowthMacroPhase::Mature => "veg_clean_day".into(),
            RegrowthMacroPhase::None if stage == SuccessionTopologyStage::BurnScar => {
                "veg_damaged".into()
            }
            RegrowthMacroPhase::None => "veg_clean_day".into(),
        };
    }
    match stage {
        SuccessionTopologyStage::BurnScar => "veg_damaged".into(),
        SuccessionTopologyStage::OldGrowth => "veg_old_growth".into(),
        _ => "veg_clean_day".into(),
    }
}

#[must_use]
pub fn planning_glyph_for_burn(
    burn: Option<&ActiveBurn>,
    stage: SuccessionTopologyStage,
) -> char {
    if burn.is_some_and(|b| b.heat > ACTIVE_BURN_HEAT_EPS) {
        return '⊗';
    }
    if let Some(b) = burn {
        return match b.regrowth_macro_phase {
            RegrowthMacroPhase::Scar => '○',
            RegrowthMacroPhase::Nuclei => '◇',
            RegrowthMacroPhase::Front => '⊕',
            RegrowthMacroPhase::Closing => '▓',
            RegrowthMacroPhase::Mature => '█',
            RegrowthMacroPhase::None => '○',
        };
    }
    match stage {
        SuccessionTopologyStage::BurnScar => '@',
        SuccessionTopologyStage::OldGrowth => '=',
        _ => '#',
    }
}

#[must_use]
pub fn extract_glyph_for_burn(
    burn: Option<&ActiveBurn>,
    stage: SuccessionTopologyStage,
) -> char {
    if burn.is_some_and(|b| b.heat > ACTIVE_BURN_HEAT_EPS) {
        return '@';
    }
    if let Some(b) = burn {
        return match b.regrowth_macro_phase {
            RegrowthMacroPhase::Scar => '%',
            RegrowthMacroPhase::Nuclei => '*',
            RegrowthMacroPhase::Front => '^',
            RegrowthMacroPhase::Closing => '#',
            RegrowthMacroPhase::Mature => '=',
            RegrowthMacroPhase::None => '%',
        };
    }
    match stage {
        SuccessionTopologyStage::BurnScar => '%',
        SuccessionTopologyStage::OldGrowth => '=',
        _ => '#',
    }
}

fn macro_phase_after_ticks(ticks_since_cool: u64) -> RegrowthMacroPhase {
    match ticks_since_cool {
        0..=29 => RegrowthMacroPhase::Scar,
        30..=59 => RegrowthMacroPhase::Nuclei,
        60..=89 => RegrowthMacroPhase::Front,
        90..=119 => RegrowthMacroPhase::Closing,
        _ => RegrowthMacroPhase::Mature,
    }
}

pub fn apply_active_burn_from_surface_fire(
    tick: Res<SimTick>,
    mut commands: Commands,
    mut witness: ResMut<LandscapeBurnWitness>,
    mut burning: Query<(Entity, Option<&ChunkSurfaceFire>, &mut ActiveBurn)>,
    ignite: Query<(Entity, &ChunkSurfaceFire), (With<DisturbanceHistory>, Without<ActiveBurn>)>,
) {
    witness.active_burn_chunks = 0;
    for (_entity, fire, mut burn) in burning.iter_mut() {
        let heat = fire.map(|f| f.heat).unwrap_or(0.0);
        burn.heat = heat;
        if heat > ACTIVE_BURN_HEAT_EPS {
            burn.cooled_at_tick = None;
            burn.frame_index = veg_burn_frame_index(tick.0, burn.started_tick);
            burn.severity = heat.max(burn.severity);
            if burn.regrowth_macro_phase != RegrowthMacroPhase::None && heat > ACTIVE_BURN_IGNITE_HEAT {
                burn.regrowth_macro_phase = RegrowthMacroPhase::None;
            }
        } else if burn.cooled_at_tick.is_none() {
            burn.cooled_at_tick = Some(tick.0);
            if burn.regrowth_macro_phase == RegrowthMacroPhase::None {
                burn.regrowth_macro_phase = RegrowthMacroPhase::Scar;
            }
        }
        witness.active_burn_chunks = witness.active_burn_chunks.saturating_add(1);
    }
    for (entity, fire) in &ignite {
        if fire.heat > ACTIVE_BURN_IGNITE_HEAT {
            commands.entity(entity).insert(ActiveBurn {
                heat: fire.heat,
                frame_index: 0,
                started_tick: tick.0,
                severity: fire.heat,
                regrowth_macro_phase: RegrowthMacroPhase::None,
                cooled_at_tick: None,
            });
            witness.active_burn_chunks = witness.active_burn_chunks.saturating_add(1);
        }
    }
}

pub fn advance_regrowth_macro_chain(
    tick: Res<SimTick>,
    mut q: Query<&mut ActiveBurn>,
    mut witness: ResMut<LandscapeBurnWitness>,
) {
    for mut burn in &mut q {
        if burn.heat > ACTIVE_BURN_HEAT_EPS {
            continue;
        }
        let Some(cooled_at) = burn.cooled_at_tick else {
            continue;
        };
        let since = tick.0.saturating_sub(cooled_at);
        let next = macro_phase_after_ticks(since);
        if next != burn.regrowth_macro_phase {
            burn.regrowth_macro_phase = next;
            witness.regrowth_macro_advances = witness.regrowth_macro_advances.saturating_add(1);
        }
    }
}

pub fn remove_mature_active_burn_overlays(
    mut commands: Commands,
    mut witness: ResMut<LandscapeBurnWitness>,
    q: Query<(Entity, &ActiveBurn)>,
) {
    for (entity, burn) in &q {
        if burn.heat <= ACTIVE_BURN_HEAT_EPS
            && burn.regrowth_macro_phase == RegrowthMacroPhase::Mature
        {
            commands.entity(entity).remove::<ActiveBurn>();
            witness.overlay_removals = witness.overlay_removals.saturating_add(1);
        }
    }
}

#[must_use]
pub fn burn_overlay_witness_green(witness: &LandscapeBurnWitness) -> bool {
    witness.active_burn_chunks >= 1
}

#[must_use]
pub fn burn_succession_witness_green(
    lg2: &LandscapeGrammarLg2Witness,
    burn: &LandscapeBurnWitness,
) -> bool {
    lg2.fire_disturbances >= 1 && burn.regrowth_macro_advances >= 1
}

#[must_use]
pub fn refresh_burn_overlay_witness(
    lg2: &LandscapeGrammarLg2Witness,
    burn: &LandscapeBurnWitness,
) -> bool {
    let body = json!({
        "gate": "VEG-BURN-OVERLAY-001",
        "green": burn_overlay_witness_green(burn),
        "active_burn_chunks": burn.active_burn_chunks,
        "regrowth_macro_advances": burn.regrowth_macro_advances,
        "overlay_removals": burn.overlay_removals,
        "fire_disturbances": lg2.fire_disturbances,
        "succession_003_green": burn_succession_witness_green(lg2, burn),
    });
    let wrapped = wrap_debug_run(
        "VEG-BURN-OVERLAY-001",
        "refresh_burn_overlay_witness",
        LANDSCAPE_GRAMMAR_BURN_OVERLAY_LIVE_JSON,
        body,
    );
    write_debug_run_json(LANDSCAPE_GRAMMAR_BURN_OVERLAY_LIVE_JSON, wrapped);
    burn_overlay_witness_green(burn)
}

#[must_use]
pub fn burn_sm_self_check_green() -> bool {
    use crate::systems::sim_control::SimTick;
    use std::collections::VecDeque;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<SimTick>()
        .init_resource::<LandscapeGrammarLg2Witness>()
        .init_resource::<LandscapeBurnWitness>()
        .add_systems(
            Update,
            (
                super::landscape_grammar_lg2::apply_fire_disturbance_on_heat,
                apply_active_burn_from_surface_fire,
                advance_regrowth_macro_chain,
            )
                .chain(),
        );

    let entity = app
        .world_mut()
        .spawn((
            Chunk {
                coord: super::landscape_grammar::LG1_PILOT_CHUNK,
            },
            ChunkSurfaceFire {
                heat: 0.9,
                fuel: 0.5,
            },
            SuccessionState {
                age_ticks: 500,
                stage: SuccessionTopologyStage::OldGrowth,
                last_disturbance_tick: None,
            },
            DisturbanceHistory {
                events: VecDeque::new(),
                capacity: 8,
            },
        ))
        .id();

    app.update();
    app.world().get::<ActiveBurn>(entity).is_some()
}

pub fn landscape_grammar_burn_plugin(app: &mut App) {
    app.init_resource::<LandscapeBurnWitness>()
        .configure_sets(
            Update,
            (
                LandscapeBurnSet::ApplyActiveBurn,
                LandscapeBurnSet::AdvanceRegrowthMacro.after(LandscapeBurnSet::ApplyActiveBurn),
                LandscapeBurnSet::RemoveMatureOverlay.after(LandscapeBurnSet::AdvanceRegrowthMacro),
            ),
        )
        .add_systems(
            Update,
            (
                apply_active_burn_from_surface_fire.in_set(LandscapeBurnSet::ApplyActiveBurn),
                advance_regrowth_macro_chain.in_set(LandscapeBurnSet::AdvanceRegrowthMacro),
                remove_mature_active_burn_overlays.in_set(LandscapeBurnSet::RemoveMatureOverlay),
            )
                .chain()
                .in_set(ChunkEnvironmentSet::Ecology),
        );
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::landscape_grammar::LG1_PILOT_CHUNK;
    use super::super::landscape_grammar_lg2::apply_fire_disturbance_on_heat;
    use crate::systems::fire::ChunkSurfaceFire;
    use crate::terrain::generation::Chunk;
    use std::collections::VecDeque;

    #[test]
    fn burn_sm_self_check_green() {
        assert!(super::burn_sm_self_check_green());
    }

    #[test]
    fn burn_frame_determinism() {
        let a = veg_burn_frame_index(100, 0);
        let b = veg_burn_frame_index(100, 0);
        assert_eq!(a, b);
        let c = veg_burn_frame_index(100 + u64::from(VEG_BURN_FRAME_PERIOD_MS / 16), 0);
        assert_ne!(a, c);
    }

    #[test]
    fn ignite_cool_and_macro_regrowth() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SimTick>()
            .init_resource::<LandscapeGrammarLg2Witness>()
            .init_resource::<LandscapeBurnWitness>()
            .add_systems(
                Update,
                (
                    apply_fire_disturbance_on_heat,
                    apply_active_burn_from_surface_fire,
                    advance_regrowth_macro_chain,
                )
                    .chain(),
            );

        let entity = app
            .world_mut()
            .spawn((
                Chunk {
                    coord: LG1_PILOT_CHUNK,
                },
                ChunkSurfaceFire {
                    heat: 0.9,
                    fuel: 0.5,
                },
                SuccessionState {
                    age_ticks: 500,
                    stage: SuccessionTopologyStage::OldGrowth,
                    last_disturbance_tick: None,
                },
                DisturbanceHistory {
                    events: VecDeque::new(),
                    capacity: 8,
                },
            ))
            .id();

        app.update();
        assert!(app.world().get::<ActiveBurn>(entity).is_some());

        app.world_mut().resource_mut::<SimTick>().0 = 50;
        {
            let mut fire = app.world_mut().get_mut::<ChunkSurfaceFire>(entity).unwrap();
            fire.heat = 0.0;
        }
        app.update();
        app.update();

        let burn = app.world().get::<ActiveBurn>(entity).unwrap();
        assert!(burn.heat <= ACTIVE_BURN_HEAT_EPS);
        assert_eq!(burn.regrowth_macro_phase, RegrowthMacroPhase::Scar);

        app.world_mut().resource_mut::<SimTick>().0 = 90;
        app.update();
        let burn = app.world().get::<ActiveBurn>(entity).unwrap();
        assert_eq!(burn.regrowth_macro_phase, RegrowthMacroPhase::Nuclei);
    }

    #[test]
    fn extract_glyph_deterministic() {
        let burn = ActiveBurn {
            heat: 0.8,
            frame_index: 2,
            started_tick: 0,
            severity: 0.8,
            regrowth_macro_phase: RegrowthMacroPhase::None,
            cooled_at_tick: None,
        };
        let a = extract_glyph_for_burn(Some(&burn), SuccessionTopologyStage::OldGrowth);
        let b = extract_glyph_for_burn(Some(&burn), SuccessionTopologyStage::OldGrowth);
        assert_eq!(a, b);
        assert_eq!(a, '@');
        assert!(variant_key_for_burn_row(Some(&burn), SuccessionTopologyStage::OldGrowth)
            .starts_with("veg_burn_"));
    }
}
