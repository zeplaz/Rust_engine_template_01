//! Destruction cause + staged collapse (plan_destruction_presentation_v1 D-6/D-9).
//!
//! * [`DestroyCause`] is threaded through the one demolish funnel
//!   ([`super::execute_demolish_at_tile`]); the funnel publishes a [`SiteDestroyed`] message per
//!   site it removes. Rubble, collapse dust and burning rubble consume that message instead of
//!   guessing from a vanished component, so an undone placement or a scenario teardown (plain
//!   despawns, no message) and a cancelled build (`PlayerDemolish` of a site that never stood)
//!   leave nothing behind.
//! * [`CollapsePending`] is the short visible collapse of a blast at or over the gate: the site
//!   sits at [`DestructibleDamageStage::Collapsed`] (broken columns, dust) for
//!   [`BLAST_COLLAPSE_MICROS`] of sim time, then [`drain_collapse_pending_system`] runs the funnel.
//!   No second despawn path: the drain only calls `execute_demolish_at_tile`.
//! * [`SiteMaterial`] is the one material vocabulary for what a destroyed site leaves (rubble bulk,
//!   dust tint).

use bevy::prelude::*;

use crate::economy::activation::BuildingDefinitionRef;
use crate::strategic::{
    BuildSiteTile, ConstructionSite, PlannedSite, SiteArchetype, SiteConstructionPhase,
    SiteFootprint, TileOccupationBook, VoxelOccupationBook, VoxelOccupationConfig,
};
use crate::systems::sim_control::SimStepClock;

use super::demolish::execute_demolish_at_tile;

/// Sim time (µs) a blasted site shows its collapse before it is demolished.
pub const BLAST_COLLAPSE_MICROS: u64 = 1_500_000;

/// Why a site left the world. Only real destruction leaves wreckage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DestroyCause {
    /// Blast at or over the demolish gate (`source_flight` for the hydrology dirty emit).
    Blast { source_flight: u64 },
    /// Fire integrity ran out.
    Fire,
    /// Flood / structural cascade (foundation failure).
    Structural,
    /// Player demolish tool (and its redo). Demolishing a build still in progress is a cancel.
    PlayerDemolish,
}

impl DestroyCause {
    /// True when this removal of a site in `phase` leaves rubble and dust.
    #[must_use]
    pub const fn leaves_wreckage(self, phase: SiteConstructionPhase) -> bool {
        use SiteConstructionPhase as P;
        match self {
            // Something was physically there once construction started.
            Self::Blast { .. } | Self::Fire | Self::Structural => matches!(
                phase,
                P::UnderConstruction | P::Provisioning | P::Operational | P::Damaged | P::Offline
            ),
            // Demolishing a standing building is destruction; cancelling a build in progress is not.
            Self::PlayerDemolish => {
                matches!(phase, P::Provisioning | P::Operational | P::Damaged | P::Offline)
            }
        }
    }
}

/// What a structure is built of (drives rubble bulk and dust tint).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SiteMaterial {
    Concrete,
    Brick,
    Timber,
    Earth,
}

impl SiteMaterial {
    /// Material of a site's category (catalog archetype): the dam is an earth embankment,
    /// fortifications and heavy plant are concrete, light industry is brick, housing and light
    /// posts are timber.
    #[must_use]
    pub const fn of_archetype(a: SiteArchetype) -> Self {
        match a {
            SiteArchetype::Dam => Self::Earth,
            SiteArchetype::MilitaryBase
            | SiteArchetype::BunkerComplex
            | SiteArchetype::DefensiveWall
            | SiteArchetype::DragonTeeth
            | SiteArchetype::PowerPlant => Self::Concrete,
            SiteArchetype::Factory
            | SiteArchetype::RailDepot
            | SiteArchetype::WaterPlant
            | SiteArchetype::FuelDepot => Self::Brick,
            SiteArchetype::CivilHousing
            | SiteArchetype::RadarSite
            | SiteArchetype::SensorPost
            | SiteArchetype::TrenchLine
            | SiteArchetype::Minefield => Self::Timber,
        }
    }

    /// Share of the standing envelope that ends up as solid rubble (rooms collapse, rubble bulks).
    #[must_use]
    pub const fn rubble_solid_fraction(self) -> f32 {
        match self {
            Self::Concrete => 0.35,
            Self::Brick => 0.30,
            Self::Timber => 0.18,
            Self::Earth => 0.60,
        }
    }

    /// Smoke-tint id carried in particle rows (`render::fire_vfx::pack::encode_smoke_tint`);
    /// 0 is plain grey smoke.
    #[must_use]
    pub const fn tint_id(self) -> u8 {
        match self {
            Self::Concrete => 1,
            Self::Brick => 2,
            Self::Timber => 3,
            Self::Earth => 4,
        }
    }

    #[must_use]
    pub const fn from_tint_id(id: u8) -> Option<Self> {
        match id {
            1 => Some(Self::Concrete),
            2 => Some(Self::Brick),
            3 => Some(Self::Timber),
            4 => Some(Self::Earth),
            _ => None,
        }
    }

    /// Dust colour (linear-ish RGB, dim: smoke blends additively). Mirrored in
    /// `fire_particle_draw.wgsl`.
    #[must_use]
    pub const fn dust_rgb(self) -> [f32; 3] {
        match self {
            Self::Concrete => [0.36, 0.36, 0.37],
            Self::Brick => [0.46, 0.27, 0.21],
            Self::Timber => [0.37, 0.28, 0.17],
            Self::Earth => [0.41, 0.33, 0.23],
        }
    }
}

/// Published by the demolish funnel for every site it removes.
#[derive(Message, Clone, Debug)]
pub struct SiteDestroyed {
    pub entity: Entity,
    pub origin: BuildSiteTile,
    pub tiles: Vec<IVec2>,
    /// Standing height (m) from the archetype; 0 for dams (their column integrity decides).
    pub height_m: f32,
    pub material: SiteMaterial,
    pub cause: DestroyCause,
    pub phase: SiteConstructionPhase,
}

impl SiteDestroyed {
    #[must_use]
    pub fn leaves_wreckage(&self) -> bool {
        self.cause.leaves_wreckage(self.phase)
    }
}

/// A site in its visible collapse; [`drain_collapse_pending_system`] demolishes it when the
/// sim-time budget is spent.
#[derive(Component, Clone, Copy, Debug)]
pub struct CollapsePending {
    pub remaining_micros: u64,
    pub cause: DestroyCause,
}

impl CollapsePending {
    #[must_use]
    pub const fn blast(source_flight: u64) -> Self {
        Self {
            remaining_micros: BLAST_COLLAPSE_MICROS,
            cause: DestroyCause::Blast { source_flight },
        }
    }
}

/// Count the pending collapse down on the sim clock and run the funnel for the ones that are due.
pub fn drain_collapse_pending_system(
    mut commands: Commands,
    clock: Res<SimStepClock>,
    mut pending: Query<(Entity, &PlannedSite, &mut CollapsePending)>,
    sites: Query<(
        Entity,
        &ConstructionSite,
        &PlannedSite,
        &SiteFootprint,
        Option<&BuildingDefinitionRef>,
    )>,
    mut occupation: Option<ResMut<TileOccupationBook>>,
    mut voxel_occupation: Option<ResMut<VoxelOccupationBook>>,
    voxel_cfg: Option<Res<VoxelOccupationConfig>>,
    mut destruction_mask: Option<ResMut<crate::render::pixel_world::DestructionMaskBook>>,
) {
    let dt = clock.step_micros.saturating_mul(u64::from(clock.due));
    if dt == 0 {
        return;
    }
    let commit_auth = voxel_cfg.as_ref().is_some_and(|c| c.commit_authority_enabled);
    let mut due: Vec<(Entity, BuildSiteTile, DestroyCause)> = Vec::new();
    for (e, planned, mut p) in &mut pending {
        p.remaining_micros = p.remaining_micros.saturating_sub(dt);
        if p.remaining_micros == 0 {
            due.push((e, planned.origin, p.cause));
        }
    }
    due.sort_by_key(|(e, o, _)| (o.x, o.z, *e));
    for (e, origin, cause) in due {
        let (n, _) = execute_demolish_at_tile(
            &mut commands,
            origin,
            &sites,
            occupation.as_deref_mut(),
            voxel_occupation.as_deref_mut(),
            commit_auth,
            destruction_mask.as_deref_mut(),
            cause,
        );
        if n == 0 {
            // Nothing covers the origin tile any more: do not leave a pending that never fires.
            commands.entity(e).remove::<CollapsePending>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::blast::{execute_blast_intents_system, BlastCraterBook, BlastIntent, BlastIntentQueue, BlastProfile};
    use crate::construction::{DestructibleDamage, DestructibleDamageStage};
    use crate::strategic::{FootprintTiles, LayerType, SiteId};

    fn spawn_standing_site(world: &mut World, tile: IVec2, phase: SiteConstructionPhase) -> Entity {
        let origin = BuildSiteTile { x: tile.x as u32, z: tile.y as u32 };
        world
            .spawn((
                ConstructionSite {
                    site_id: 5,
                    owner: Entity::PLACEHOLDER,
                    archetype: SiteArchetype::Factory,
                    phase,
                    operational_readiness: 1.0,
                },
                PlannedSite {
                    site_id: SiteId(5),
                    origin,
                    footprint: FootprintTiles { width: 1, depth: 1 },
                    archetype: SiteArchetype::Factory,
                    layer: LayerType::Surface,
                    catalog_id: None,
                    placement: None,
                },
                SiteFootprint { tiles: vec![tile], layer: LayerType::Surface },
            ))
            .id()
    }

    fn step_clock(micros: u64) -> SimStepClock {
        let mut c = SimStepClock::default();
        c.step_micros = micros;
        c.due = 1;
        c
    }

    /// A blast at or over the gate stages the site to `Collapsed`, keeps it standing for the
    /// collapse window (dust and broken columns draw), then the funnel despawns it.
    #[test]
    fn blast_collapse_shows_collapsed_before_the_despawn() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<BlastIntentQueue>()
            .init_resource::<BlastCraterBook>()
            .add_message::<SiteDestroyed>()
            .insert_resource(BlastProfile::default())
            .insert_resource(step_clock(500_000))
            .add_systems(
                Update,
                (execute_blast_intents_system, drain_collapse_pending_system).chain(),
            );
        let tile = IVec2::new(0, 0);
        let site = spawn_standing_site(app.world_mut(), tile, SiteConstructionPhase::Operational);
        let c = Vec2::splat(8.0);
        app.world_mut().resource_mut::<BlastIntentQueue>().enqueue(BlastIntent {
            center_xz: c,
            radius_m: 16.0,
            damage: 200.0,
            penetration_m: 0.0,
            source_flight: 3,
        });
        app.update();
        let e = app.world().get_entity(site).expect("still standing in its collapse window");
        assert_eq!(
            e.get::<DestructibleDamage>().map(|d| d.stage),
            Some(DestructibleDamageStage::Collapsed)
        );
        assert!(e.get::<CollapsePending>().is_some());
        app.update();
        assert!(app.world().get_entity(site).is_ok(), "1.0 s of 1.5 s spent");
        app.update();
        app.update();
        assert!(app.world().get_entity(site).is_err(), "demolished through the funnel after 1.5 s");
        // The funnel announced it with the blast cause.
        let msgs = app.world().resource::<Messages<SiteDestroyed>>();
        let mut cursor = msgs.get_cursor();
        let got: Vec<_> = cursor.read(msgs).collect();
        assert_eq!(got.len(), 1);
        assert!(matches!(got[0].cause, DestroyCause::Blast { source_flight: 3 }));
        assert!(got[0].leaves_wreckage());
    }

    #[test]
    fn only_real_destruction_leaves_wreckage() {
        use SiteConstructionPhase as P;
        assert!(DestroyCause::Fire.leaves_wreckage(P::Operational));
        assert!(DestroyCause::Blast { source_flight: 0 }.leaves_wreckage(P::UnderConstruction));
        assert!(DestroyCause::PlayerDemolish.leaves_wreckage(P::Operational));
        assert!(!DestroyCause::PlayerDemolish.leaves_wreckage(P::UnderConstruction));
        assert!(!DestroyCause::Fire.leaves_wreckage(P::Planned));
    }

    #[test]
    fn materials_cover_every_archetype_and_roundtrip_tint_ids() {
        assert_eq!(SiteMaterial::of_archetype(SiteArchetype::Dam), SiteMaterial::Earth);
        assert_eq!(SiteMaterial::of_archetype(SiteArchetype::BunkerComplex), SiteMaterial::Concrete);
        assert_eq!(SiteMaterial::of_archetype(SiteArchetype::CivilHousing), SiteMaterial::Timber);
        for m in [SiteMaterial::Concrete, SiteMaterial::Brick, SiteMaterial::Timber, SiteMaterial::Earth] {
            assert_eq!(SiteMaterial::from_tint_id(m.tint_id()), Some(m));
        }
        assert_eq!(SiteMaterial::from_tint_id(0), None);
    }
}
