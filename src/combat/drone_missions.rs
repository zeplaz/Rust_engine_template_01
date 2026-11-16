//! D1 — drone / air mission intent (AO + packages + MissionBook). No free-flying airframe entities.
//!
//! Player/AI **enqueue** only ([`MissionIntentQueue`]); [`step_drone_missions_system`] is the sole
//! writer of [`MissionBook`] / [`DroneCombatModifiers`]. Sorties are automated under weather
//! envelopes; presentation hints are additive for D8 fidelity upgrades.
//!
//! Plan: [`plan_drone_subsystem_fit_v1.md`](../dev/plan_drone_subsystem_fit_v1.md).

use std::collections::{BTreeMap, HashMap, VecDeque};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::capability_toe::MissileClass;
use super::drone_doctrine::{doctrine_effect_scale, FactionDroneDoctrine};
use super::drone_research::DroneResearchState;
use super::formation::{CombatClock, Formation};
use super::missile_supply::{pulse_missile_sorties, MissilePackage, MissileSortieDump};
use crate::substrate::atmosphere::AtmosphereClipmapStack;
use crate::systems::fire::ChunkSmokeField;
use crate::systems::weather::{
    collect_chunk_smoke_density, evaluate_grounding, smoke01_at_tile, sortie_envelopes_ok,
    AirOpsProfileId, AirOpsTuning, AirPresentationSortieHint, LinkModality, SensorKind,
    WeatherPollingStations, SMOKE_SAMPLE_CELLS_PER_CHUNK,
};
use crate::terrain::generation::Chunk;

/// Drone role in a formation package / AO (RON/data identity).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DroneRole {
    FpvStrike = 0,
    ReconLoiter = 1,
    AttritCruise = 2,
    MedAttack = 3,
    Interceptor = 4,
    SwarmMember = 5,
}

impl DroneRole {
    pub const ALL: [DroneRole; 6] = [
        DroneRole::FpvStrike,
        DroneRole::ReconLoiter,
        DroneRole::AttritCruise,
        DroneRole::MedAttack,
        DroneRole::Interceptor,
        DroneRole::SwarmMember,
    ];

    #[must_use]
    pub fn index(self) -> usize {
        self as usize
    }

    #[must_use]
    pub fn air_ops_profile(self) -> AirOpsProfileId {
        match self {
            DroneRole::FpvStrike | DroneRole::SwarmMember => AirOpsProfileId::DroneFpv,
            DroneRole::ReconLoiter | DroneRole::MedAttack => AirOpsProfileId::DroneRecon,
            DroneRole::AttritCruise => AirOpsProfileId::DroneCruise,
            DroneRole::Interceptor => AirOpsProfileId::DroneInterceptor,
        }
    }

    #[must_use]
    pub fn is_mass(self) -> bool {
        matches!(self, DroneRole::FpvStrike | DroneRole::SwarmMember)
    }
}

/// Mission class for a [`DroneAo`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DroneMissionKind {
    Hunt,
    Recon,
    Strike,
    CuasScreen,
    SwarmSaturation,
}

/// Formation-local stock of drones by role (not airframe entities).
#[derive(Component, Clone, Debug, PartialEq)]
pub struct DronePackage {
    pub counts: [u32; 6],
    pub modality: LinkModality,
    pub sensor: SensorKind,
    /// D2: 0 = remote only, 1–3 autonomy (capped by research).
    pub autonomy_level: u8,
}

impl Default for DronePackage {
    fn default() -> Self {
        Self {
            counts: [0; 6],
            modality: LinkModality::Radio,
            sensor: SensorKind::Eo,
            autonomy_level: 0,
        }
    }
}

impl DronePackage {
    #[must_use]
    pub fn with_count(role: DroneRole, n: u32) -> Self {
        let mut p = Self::default();
        p.counts[role.index()] = n;
        p
    }

    #[must_use]
    pub fn count(&self, role: DroneRole) -> u32 {
        self.counts[role.index()]
    }

    #[must_use]
    pub fn total(&self) -> u32 {
        self.counts.iter().sum()
    }

    /// Consume up to `n` of `role`; returns how many were taken.
    pub fn take(&mut self, role: DroneRole, n: u32) -> u32 {
        let i = role.index();
        let got = self.counts[i].min(n);
        self.counts[i] -= got;
        got
    }
}

/// Optional air-ops zone (AABB tiles). Intent surface for manned/pool air later.
#[derive(Clone, Debug, PartialEq)]
pub struct AirOpZone {
    pub id: u64,
    pub faction_slot: u8,
    pub min: IVec2,
    pub max: IVec2,
    pub priority: f32,
    pub paused: bool,
}

impl AirOpZone {
    #[must_use]
    pub fn centroid_tile(&self) -> IVec2 {
        IVec2::new((self.min.x + self.max.x) / 2, (self.min.y + self.max.y) / 2)
    }
}

/// Drone area of operations + mission type / primary role (D1: one role per AO).
#[derive(Clone, Debug, PartialEq)]
pub struct DroneAo {
    pub id: u64,
    pub faction_slot: u8,
    pub min: IVec2,
    pub max: IVec2,
    pub mission: DroneMissionKind,
    pub role: DroneRole,
    pub priority: f32,
    pub weight: f32,
    pub paused: bool,
}

impl DroneAo {
    #[must_use]
    pub fn centroid_tile(&self) -> IVec2 {
        IVec2::new((self.min.x + self.max.x) / 2, (self.min.y + self.max.y) / 2)
    }
}


/// TAC-11: script a cooperative multi-role package as enqueue intents (recon screen + strike).
/// Deterministic ids: `base_id` / `base_id+1`. Does not mutate the queue — caller enqueues.
#[must_use]
pub fn script_group_package_intents(
    faction_slot: u8,
    base_id: u64,
    area_min: IVec2,
    area_max: IVec2,
) -> Vec<MissionIntent> {
    let recon = DroneAo {
        id: base_id,
        faction_slot,
        min: area_min,
        max: area_max,
        mission: DroneMissionKind::Recon,
        role: DroneRole::ReconLoiter,
        priority: 0.6,
        weight: 1.0,
        paused: false,
    };
    let strike = DroneAo {
        id: base_id.saturating_add(1),
        faction_slot,
        min: area_min,
        max: area_max,
        mission: DroneMissionKind::Strike,
        role: DroneRole::FpvStrike,
        priority: 0.8,
        weight: 1.0,
        paused: false,
    };
    vec![
        MissionIntent::UpsertDroneAo(recon),
        MissionIntent::UpsertDroneAo(strike),
    ]
}

/// Enqueue-only intents (player / AI).
#[derive(Clone, Debug, PartialEq)]
pub enum MissionIntent {
    UpsertAirOpZone(AirOpZone),
    UpsertDroneAo(DroneAo),
    RemoveAirOpZone { id: u64 },
    RemoveDroneAo { id: u64 },
    PauseDroneAo { id: u64, paused: bool },
    /// Supply share 0..1 for an AO (distribution lever; D4 consumes later).
    SetSupplyShare { faction_slot: u8, ao_id: u64, share: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissionEnqueueError {
    QueueFull,
}

pub type MissionIntentId = u64;

#[derive(Resource, Clone, Debug)]
pub struct MissionIntentQueue {
    pending: VecDeque<(MissionIntentId, MissionIntent)>,
    next_id: MissionIntentId,
    pub capacity: usize,
}

impl Default for MissionIntentQueue {
    fn default() -> Self {
        Self {
            pending: VecDeque::new(),
            next_id: 1,
            capacity: 4096,
        }
    }
}

impl MissionIntentQueue {
    pub fn enqueue(&mut self, intent: MissionIntent) -> Result<MissionIntentId, MissionEnqueueError> {
        if self.pending.len() >= self.capacity {
            return Err(MissionEnqueueError::QueueFull);
        }
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.pending.push_back((id, intent));
        Ok(id)
    }

    pub fn drain(&mut self) -> Vec<(MissionIntentId, MissionIntent)> {
        self.pending.drain(..).collect()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

/// Combat modifiers from successful sorties this pulse (sole writer: mission step).
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct DroneCombatModifiers {
    pub recon_detect_mul: f32,
    pub fpv_firepower_mul: f32,
    /// Soft threat 0..1 after C-UAS reduction.
    pub cruise_threat: f32,
    /// D3: successful interceptor / CUAS screen sorties this pulse.
    pub cuas_intercepts: u32,
}

impl Default for DroneCombatModifiers {
    fn default() -> Self {
        Self {
            recon_detect_mul: 1.0,
            fpv_firepower_mul: 1.0,
            cruise_threat: 0.0,
            cuas_intercepts: 0,
        }
    }
}

/// Dump / matrix counters.
#[derive(Resource, Debug, Clone, Default)]
pub struct DroneMissionDump {
    pub missions_active: u32,
    pub sorties_aborted_wx: u64,
    pub sorties_ok: u64,
    pub ao_count: u32,
    pub samples: u64,
}

/// Sim-owned mission state (sole writer: [`step_drone_missions_system`]).
#[derive(Resource, Clone, Debug, Default)]
pub struct MissionBook {
    pub air_op_zones: BTreeMap<u64, AirOpZone>,
    pub drone_aos: BTreeMap<u64, DroneAo>,
    pub supply_share: BTreeMap<(u8, u64), f32>,
    pub presentation_hints: Vec<AirPresentationSortieHint>,
    pub sorties_ok: u64,
    pub sorties_aborted_wx: u64,
    pub(crate) sortie_acc_s: f32,
}

impl MissionBook {
    #[must_use]
    pub fn active_mission_count(&self) -> u32 {
        self.drone_aos.values().filter(|a| !a.paused).count() as u32
            + self.air_op_zones.values().filter(|z| !z.paused).count() as u32
    }
}

const SORTIE_PULSE_S: f32 = 5.0;
pub(crate) const LINK_FLOOR: f32 = 0.35;
pub(crate) const SENSOR_FLOOR: f32 = 0.35;

fn apply_intents(book: &mut MissionBook, intents: Vec<(MissionIntentId, MissionIntent)>) {
    for (_, intent) in intents {
        match intent {
            MissionIntent::UpsertAirOpZone(z) => {
                book.air_op_zones.insert(z.id, z);
            }
            MissionIntent::UpsertDroneAo(a) => {
                book.drone_aos.insert(a.id, a);
            }
            MissionIntent::RemoveAirOpZone { id } => {
                book.air_op_zones.remove(&id);
            }
            MissionIntent::RemoveDroneAo { id } => {
                book.drone_aos.remove(&id);
                book.supply_share.retain(|&(_, ao), _| ao != id);
            }
            MissionIntent::PauseDroneAo { id, paused } => {
                if let Some(a) = book.drone_aos.get_mut(&id) {
                    a.paused = paused;
                }
            }
            MissionIntent::SetSupplyShare {
                faction_slot,
                ao_id,
                share,
            } => {
                book.supply_share
                    .insert((faction_slot, ao_id), share.clamp(0.0, 1.0));
            }
        }
    }
}

fn default_modality(role: DroneRole) -> LinkModality {
    match role {
        DroneRole::FpvStrike | DroneRole::SwarmMember => LinkModality::Radio,
        DroneRole::ReconLoiter => LinkModality::SatRelay,
        DroneRole::AttritCruise => LinkModality::Autonomy,
        DroneRole::MedAttack | DroneRole::Interceptor => LinkModality::Radio,
    }
}

fn effective_modality(
    pkg: &DronePackage,
    role: DroneRole,
    research: &DroneResearchState,
) -> LinkModality {
    let mut m = pkg.modality;
    if matches!(m, LinkModality::Fiber) && !research.fiber_ops {
        m = LinkModality::Radio;
    }
    let auto_cap = research.max_autonomy_level();
    let level = pkg.autonomy_level.min(auto_cap);
    if level >= 2 {
        m = LinkModality::Autonomy;
    } else if matches!(role, DroneRole::AttritCruise) && level >= 1 {
        m = LinkModality::Autonomy;
    }
    let _ = default_modality(role);
    m
}

fn effective_sensor(pkg: &DronePackage, research: &DroneResearchState) -> SensorKind {
    match pkg.sensor {
        SensorKind::Ir if !research.seeker_ir_unlock => SensorKind::Eo,
        other => other,
    }
}

/// Pulse sorties for all active drone AOs (pure-ish core for tests + system).
pub fn pulse_drone_sorties(
    book: &mut MissionBook,
    mods: &mut DroneCombatModifiers,
    packages: &mut [(Entity, &Formation, &mut DronePackage)],
    tuning: &AirOpsTuning,
    stations: Option<&WeatherPollingStations>,
    research: &DroneResearchState,
    doctrine: &FactionDroneDoctrine,
    smoke_by_chunk: Option<&HashMap<IVec2, f32>>,
    atmos: Option<&AtmosphereClipmapStack>,
    faction_allowed: &dyn Fn(u8) -> bool,
) {
    *mods = DroneCombatModifiers::default();
    book.presentation_hints.clear();

    let mut aos: Vec<DroneAo> = book.drone_aos.values().cloned().collect();
    aos.sort_by(|a, b| {
        b.priority
            .partial_cmp(&a.priority)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });

    let cells = UVec2::splat(SMOKE_SAMPLE_CELLS_PER_CHUNK);
    let empty_smoke = HashMap::new();
    let smoke_map = smoke_by_chunk.unwrap_or(&empty_smoke);

    let mut recon_ok = 0u32;
    let mut fpv_ok = 0u32;
    let mut cruise_ok = 0u32;
    let mut cuas_ok = 0u32;

    for ao in aos {
        if ao.paused || !faction_allowed(ao.faction_slot) {
            continue;
        }
        if matches!(ao.role, DroneRole::AttritCruise) && !research.cruise_line_unlock {
            // Allow stock use once unlocked; until then treat as med attack profile still ok —
            // gate only blocks doctrine saturation cruise lines (soft: still fly with recon profile).
        }
        let c = ao.centroid_tile();
        let mut best: Option<(usize, i32)> = None;
        for (i, (_, f, p)) in packages.iter().enumerate() {
            if f.faction_slot != ao.faction_slot || p.count(ao.role) == 0 {
                continue;
            }
            let d = (f.tile - c).abs().x + (f.tile - c).abs().y;
            if best.is_none_or(|(_, bd)| d < bd) {
                best = Some((i, d));
            }
        }
        let Some((idx, _)) = best else {
            continue;
        };
        let (_, _f, pkg) = &mut packages[idx];
        let modality = effective_modality(pkg, ao.role, research);
        let sensor = effective_sensor(pkg, research);
        let profile = tuning.profile_or_default(ao.role.air_ops_profile());
        let sample = stations
            .filter(|s| s.ready)
            .map(|s| s.sample_at_tile(c))
            .unwrap_or_default();
        let smoke01 = smoke01_at_tile(c, cells, smoke_map, atmos);
        // D2: research resist into envelope floors (implemented via higher effective resist in checks).
        let link_r = research.link_resist(profile.link_resist);
        let sens_r = research.sensor_resist(profile.sensor_resist);
        let mut gated_profile = profile.clone();
        gated_profile.link_resist = link_r;
        gated_profile.sensor_resist = sens_r;
        let ok = sortie_envelopes_ok(
            &sample,
            &gated_profile,
            smoke01,
            modality,
            sensor,
            LINK_FLOOR,
            SENSOR_FLOOR,
        ) && !evaluate_grounding(&sample, &gated_profile, None).is_grounded();

        let wind = sample.wind_mps;
        let heading = if wind.length_squared() > 1e-4 {
            wind.normalize()
        } else {
            Vec2::X
        };
        book.presentation_hints.push(AirPresentationSortieHint {
            mission_id: ao.id,
            role_mass: ao.role.is_mass(),
            wind_mps: wind,
            heading,
            path_samples_m: Vec::new(),
            sortie_tick: book.sorties_ok.wrapping_add(book.sorties_aborted_wx),
        });

        if !ok {
            book.sorties_aborted_wx = book.sorties_aborted_wx.wrapping_add(1);
            continue;
        }

        let expend = matches!(
            ao.role,
            DroneRole::FpvStrike
                | DroneRole::AttritCruise
                | DroneRole::SwarmMember
                | DroneRole::MedAttack
                | DroneRole::Interceptor
        );
        if expend && pkg.take(ao.role, 1) == 0 {
            continue;
        }

        book.sorties_ok = book.sorties_ok.wrapping_add(1);
        match ao.role {
            DroneRole::ReconLoiter => recon_ok += 1,
            DroneRole::FpvStrike | DroneRole::SwarmMember => fpv_ok += 1,
            DroneRole::AttritCruise | DroneRole::MedAttack => cruise_ok += 1,
            DroneRole::Interceptor => cuas_ok += 1,
        }
        if matches!(ao.mission, DroneMissionKind::CuasScreen) {
            cuas_ok += 1;
        }
    }

    // D6: doctrine scales strike effects per first active AO faction (or slot 0).
    let doc = doctrine_for_book(book, doctrine);
    let strike_scale = doctrine_effect_scale(doc);

    if recon_ok > 0 {
        mods.recon_detect_mul = (1.0 + 0.08 * recon_ok as f32).min(1.35);
    }
    if fpv_ok > 0 {
        // Doctrine scales the *bonus*, never pulls baseline firepower below 1.
        mods.fpv_firepower_mul = (1.0 + 0.10 * fpv_ok as f32 * strike_scale).min(1.50);
    }
    let raw_cruise = if cruise_ok > 0 {
        (0.15 * cruise_ok as f32 * strike_scale).clamp(0.0, 1.0)
    } else {
        0.0
    };
    // D3: each CUAS intercept cuts cruise threat.
    mods.cuas_intercepts = cuas_ok;
    let cut = (0.12 * cuas_ok as f32).min(raw_cruise);
    mods.cruise_threat = (raw_cruise - cut).clamp(0.0, 1.0);
}

fn doctrine_for_book(
    book: &MissionBook,
    doctrine: &FactionDroneDoctrine,
) -> crate::strategic::DroneDoctrine {
    book.drone_aos
        .values()
        .next()
        .map(|a| doctrine.get(a.faction_slot))
        .unwrap_or_else(|| doctrine.get(0))
}

fn refresh_dump(book: &MissionBook, dump: &mut DroneMissionDump) {
    dump.missions_active = book.active_mission_count();
    dump.ao_count = book.drone_aos.len() as u32;
    dump.sorties_aborted_wx = book.sorties_aborted_wx;
    dump.sorties_ok = book.sorties_ok;
}

/// Bevy **0.20** forward pattern: group >16 world accesses via [`SystemParam`] derive
/// (docs.rs/bevy/0.20 — function systems still max **16 top-level args**; derive folds fields).
/// Do **not** stuff ballistics/audio `ResMut` here — enqueue only ([`MissileSortieLaunchQueue`]).
#[derive(SystemParam)]
pub struct MissionStepWriters<'w> {
    pub queue: ResMut<'w, MissionIntentQueue>,
    pub book: ResMut<'w, MissionBook>,
    pub mods: ResMut<'w, DroneCombatModifiers>,
    pub dump: ResMut<'w, DroneMissionDump>,
    pub missile_dump: ResMut<'w, MissileSortieDump>,
    pub launch_q: ResMut<'w, super::missile_supply::MissileSortieLaunchQueue>,
}

#[derive(SystemParam)]
pub struct MissionStepReads<'w> {
    pub clock: Res<'w, CombatClock>,
    pub tuning: Res<'w, AirOpsTuning>,
    pub stations: Option<Res<'w, WeatherPollingStations>>,
    pub research: Res<'w, DroneResearchState>,
    pub doctrine: Res<'w, FactionDroneDoctrine>,
    pub attention: Option<Res<'w, crate::systems::sim_attention::SimAttentionFrame>>,
    pub atmos: Option<Res<'w, AtmosphereClipmapStack>>,
}

#[derive(SystemParam)]
pub struct MissionStepQueries<'w, 's> {
    pub smoke_q: Query<'w, 's, (&'static Chunk, &'static ChunkSmokeField)>,
    pub packages: Query<'w, 's, (Entity, &'static Formation, &'static mut DronePackage)>,
    pub missile_packages:
        Query<'w, 's, (Entity, &'static Formation, &'static mut MissilePackage)>,
}

/// Sole writer of [`MissionBook`], [`DroneCombatModifiers`], [`DroneMissionDump`].
/// Also pulses P6 missile sorties (consumes [`MissilePackage`]; updates [`MissileSortieDump`]).
pub fn step_drone_missions_system(
    mut w: MissionStepWriters,
    r: MissionStepReads,
    mut q: MissionStepQueries,
) {
    apply_intents(&mut w.book, w.queue.drain());

    if r.clock.dt_s <= 0.0 {
        refresh_dump(&w.book, &mut w.dump);
        return;
    }
    w.book.sortie_acc_s += r.clock.dt_s;
    if w.book.sortie_acc_s < SORTIE_PULSE_S {
        refresh_dump(&w.book, &mut w.dump);
        return;
    }
    w.book.sortie_acc_s = 0.0;

    let allow = |slot: u8| {
        r.attention
            .as_ref()
            .map(|a| a.tier_for_faction_slot(slot).allows_sortie_pulse())
            .unwrap_or(true)
    };

    let smoke_by_chunk = collect_chunk_smoke_density(&q.smoke_q);

    let mut rows: Vec<(Entity, Formation, DronePackage)> = q
        .packages
        .iter_mut()
        .map(|(e, f, p)| (e, f.clone(), p.clone()))
        .collect();
    rows.sort_by_key(|(e, _, _)| e.to_bits());

    {
        let mut refs: Vec<(Entity, &Formation, &mut DronePackage)> = rows
            .iter_mut()
            .map(|(e, f, p)| (*e, &*f, p))
            .collect();
        pulse_drone_sorties(
            &mut w.book,
            &mut w.mods,
            &mut refs,
            &r.tuning,
            r.stations.as_deref(),
            &r.research,
            &r.doctrine,
            Some(&smoke_by_chunk),
            r.atmos.as_deref(),
            &allow,
        );
    }

    for (e, _, p) in &rows {
        if let Ok((_, _, mut live)) = q.packages.get_mut(*e) {
            *live = p.clone();
        }
    }

    // P6: AirOpZone sorties consume nearest missile magazine under weather envelope.
    let mut m_rows: Vec<(Entity, Formation, MissilePackage)> = q
        .missile_packages
        .iter_mut()
        .map(|(e, f, p)| (e, f.clone(), p.clone()))
        .collect();
    m_rows.sort_by_key(|(e, _, _)| e.to_bits());
    let mut launches = Vec::new();
    {
        let mut refs: Vec<(Entity, &Formation, &mut MissilePackage)> = m_rows
            .iter_mut()
            .map(|(e, f, p)| (*e, &*f, p))
            .collect();
        // Prefer SAM short for AD screens; cruise when no SAM left (pulse tries preferred only).
        launches.extend(pulse_missile_sorties(
            &w.book,
            &mut w.missile_dump,
            &mut refs,
            &r.tuning,
            r.stations.as_deref(),
            MissileClass::SamShort,
            &allow,
        ));
        launches.extend(pulse_missile_sorties(
            &w.book,
            &mut w.missile_dump,
            &mut refs,
            &r.tuning,
            r.stations.as_deref(),
            MissileClass::GenericCruise,
            &allow,
        ));
    }
    for (e, _, p) in &m_rows {
        if let Ok((_, _, mut live)) = q.missile_packages.get_mut(*e) {
            *live = p.clone();
        }
    }

    w.launch_q.pending.extend(launches);

    w.dump.samples = w.dump.samples.wrapping_add(1);
    refresh_dump(&w.book, &mut w.dump);
}

/// FIDELITY-1: ballistics + audio from missile sorties (separate system — SystemParam budget).
pub fn apply_missile_sortie_launches_system(
    mut launch_q: ResMut<super::missile_supply::MissileSortieLaunchQueue>,
    mut ballistics: Option<ResMut<super::ballistics::BallisticFlightBook>>,
    mut audio: Option<ResMut<crate::audio::AudioOneShotQueue>>,
) {
    if launch_q.pending.is_empty() {
        return;
    }
    let launches = std::mem::take(&mut launch_q.pending);
    for launch in launches {
        if let Some(audio) = audio.as_mut() {
            audio.push_warning(
                crate::audio::AudioOneShotKind::LaunchWarning,
                Some(launch.origin_xz),
            );
            audio.push_combat(
                crate::audio::AudioOneShotKind::MissileLaunch,
                launch.origin_xz,
            );
        }
        if let Some(book) = ballistics.as_mut() {
            book.launch_with_wind(
                launch.origin_xz,
                launch.target_xz,
                8.0,
                2.5,
                super::ballistics::BallisticKind::Missile,
                launch.wind,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::balance::UnitKind;
    use crate::combat::drone_doctrine::FactionDroneDoctrine;
    use crate::combat::drone_research::DroneResearchState;
    use crate::systems::weather::WeatherSample;

    fn ao(id: u64, role: DroneRole) -> DroneAo {
        DroneAo {
            id,
            faction_slot: 0,
            min: IVec2::new(0, 0),
            max: IVec2::new(10, 10),
            mission: DroneMissionKind::Hunt,
            role,
            priority: 1.0,
            weight: 1.0,
            paused: false,
        }
    }

    #[test]
    fn intent_upserts_ao() {
        let mut q = MissionIntentQueue::default();
        let mut book = MissionBook::default();
        q.enqueue(MissionIntent::UpsertDroneAo(ao(7, DroneRole::ReconLoiter)))
            .unwrap();
        apply_intents(&mut book, q.drain());
        assert_eq!(book.drone_aos.len(), 1);
        assert_eq!(book.active_mission_count(), 1);
    }

    #[test]
    fn calm_recon_sortie_raises_detect_mul() {
        let mut book = MissionBook::default();
        book.drone_aos.insert(1, ao(1, DroneRole::ReconLoiter));
        let mut mods = DroneCombatModifiers::default();
        let f = Formation::new(0, UnitKind::Recon, IVec2::new(5, 5));
        let mut pkg = DronePackage::with_count(DroneRole::ReconLoiter, 3);
        pkg.modality = LinkModality::SatRelay;
        let e = Entity::from_bits(1);
        let mut rows = vec![(e, &f, &mut pkg)];
        pulse_drone_sorties(
            &mut book,
            &mut mods,
            &mut rows,
            &AirOpsTuning::default(),
            None,
            &DroneResearchState::default(),
            &FactionDroneDoctrine::default(),
            None,
            None,
            &|_| true,
        );
        assert!(book.sorties_ok >= 1);
        assert!(mods.recon_detect_mul > 1.0);
        assert_eq!(pkg.count(DroneRole::ReconLoiter), 3, "recon does not expend");
        assert!(!book.presentation_hints.is_empty());
    }

    #[test]
    fn storm_sample_fails_fpv_envelopes() {
        let tuning = AirOpsTuning::default();
        let profile = tuning.profile_or_default(AirOpsProfileId::DroneFpv);
        let storm = WeatherSample {
            wind_mps: Vec2::new(25.0, 0.0),
            gust_mps: 28.0,
            visibility_m: 200.0,
            fog: 0.9,
            lightning: 0.9,
            ..Default::default()
        };
        assert!(!sortie_envelopes_ok(
            &storm,
            profile,
            0.9,
            LinkModality::Radio,
            SensorKind::Eo,
            LINK_FLOOR,
            SENSOR_FLOOR,
        ));
    }

    #[test]
    fn fpv_expends_stock_on_success() {
        let mut book = MissionBook::default();
        book.drone_aos.insert(1, ao(1, DroneRole::FpvStrike));
        let mut mods = DroneCombatModifiers::default();
        let f = Formation::new(0, UnitKind::Infantry, IVec2::new(5, 5));
        let mut pkg = DronePackage::with_count(DroneRole::FpvStrike, 2);
        let e = Entity::from_bits(3);
        let mut rows = vec![(e, &f, &mut pkg)];
        pulse_drone_sorties(
            &mut book,
            &mut mods,
            &mut rows,
            &AirOpsTuning::default(),
            None,
            &DroneResearchState::default(),
            &FactionDroneDoctrine::default(),
            None,
            None,
            &|_| true,
        );
        assert_eq!(pkg.count(DroneRole::FpvStrike), 1);
        assert!(mods.fpv_firepower_mul > 1.0);
    }

    #[test]
    fn cuas_interceptor_cuts_cruise_threat() {
        let mut book = MissionBook::default();
        book.drone_aos.insert(
            1,
            DroneAo {
                id: 1,
                faction_slot: 0,
                min: IVec2::ZERO,
                max: IVec2::new(10, 10),
                mission: DroneMissionKind::Strike,
                role: DroneRole::AttritCruise,
                priority: 1.0,
                weight: 1.0,
                paused: false,
            },
        );
        book.drone_aos.insert(
            2,
            DroneAo {
                id: 2,
                faction_slot: 0,
                min: IVec2::ZERO,
                max: IVec2::new(10, 10),
                mission: DroneMissionKind::CuasScreen,
                role: DroneRole::Interceptor,
                priority: 2.0,
                weight: 1.0,
                paused: false,
            },
        );
        let mut mods = DroneCombatModifiers::default();
        let f = Formation::new(0, UnitKind::AirDefense, IVec2::new(5, 5));
        let mut pkg = DronePackage::default();
        pkg.counts[DroneRole::AttritCruise.index()] = 2;
        pkg.counts[DroneRole::Interceptor.index()] = 2;
        let e = Entity::from_bits(9);
        let mut rows = vec![(e, &f, &mut pkg)];
        let mut research = DroneResearchState::default();
        research.cruise_line_unlock = true;
        pulse_drone_sorties(
            &mut book,
            &mut mods,
            &mut rows,
            &AirOpsTuning::default(),
            None,
            &research,
            &FactionDroneDoctrine::default(),
            None,
            None,
            &|_| true,
        );
        assert!(mods.cuas_intercepts >= 1);
    }

    #[test]
    fn dense_smoke_aborts_radio_fpv() {
        let mut book = MissionBook::default();
        book.drone_aos.insert(1, ao(1, DroneRole::FpvStrike));
        let mut mods = DroneCombatModifiers::default();
        let f = Formation::new(0, UnitKind::Infantry, IVec2::new(5, 5));
        let mut pkg = DronePackage::with_count(DroneRole::FpvStrike, 2);
        pkg.modality = LinkModality::Radio;
        pkg.sensor = SensorKind::Eo;
        let e = Entity::from_bits(11);
        let mut rows = vec![(e, &f, &mut pkg)];
        let mut smoke = HashMap::new();
        // AO centroid (5,5) → chunk (0,0) at 16 cells/chunk.
        smoke.insert(IVec2::ZERO, 1.0);
        pulse_drone_sorties(
            &mut book,
            &mut mods,
            &mut rows,
            &AirOpsTuning::default(),
            None,
            &DroneResearchState::default(),
            &FactionDroneDoctrine::default(),
            Some(&smoke),
            None,
            &|_| true,
        );
        assert!(book.sorties_aborted_wx >= 1);
        assert_eq!(pkg.count(DroneRole::FpvStrike), 2, "aborted sortie does not expend");
    }

    #[test]
    fn apply_missile_sortie_launches_fills_ballistics_and_audio() {
        use crate::audio::AudioOneShotQueue;
        use crate::combat::ballistics::BallisticFlightBook;
        use crate::combat::missile_supply::{MissileSortieLaunch, MissileSortieLaunchQueue};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<MissileSortieLaunchQueue>()
            .init_resource::<BallisticFlightBook>()
            .init_resource::<AudioOneShotQueue>()
            .add_systems(Update, apply_missile_sortie_launches_system);
        app.world_mut()
            .resource_mut::<MissileSortieLaunchQueue>()
            .pending
            .push(MissileSortieLaunch {
                faction_slot: 0,
                origin_xz: Vec2::new(1.0, 2.0),
                target_xz: Vec2::new(10.0, 12.0),
                wind: None,
            });
        app.update();
        assert!(
            app.world()
                .resource::<MissileSortieLaunchQueue>()
                .pending
                .is_empty()
        );
        assert_eq!(
            app.world().resource::<BallisticFlightBook>().flights.len(),
            1
        );
        assert!(
            !app.world()
                .resource::<AudioOneShotQueue>()
                .pending
                .is_empty()
        );
    }

    #[test]
    fn dormant_faction_gate_skips_missile_pulse() {
        let book = MissionBook::default();
        let mut dump = MissileSortieDump::default();
        let f = Formation::new(0, UnitKind::Infantry, IVec2::ZERO);
        let mut pkg = MissilePackage::default();
        let e = Entity::from_bits(3);
        let mut rows = vec![(e, &f, &mut pkg)];
        let launches = pulse_missile_sorties(
            &book,
            &mut dump,
            &mut rows,
            &AirOpsTuning::default(),
            None,
            MissileClass::SamShort,
            &|_| false,
        );
        assert!(launches.is_empty());
    }
    #[test]
    fn tac11_scripted_group_package_emits_cooperative_aos() {
        let intents = script_group_package_intents(0, 10, IVec2::new(0, 0), IVec2::new(4, 4));
        assert_eq!(intents.len(), 2);
        match &intents[0] {
            MissionIntent::UpsertDroneAo(ao) => {
                assert_eq!(ao.mission, DroneMissionKind::Recon);
                assert_eq!(ao.role, DroneRole::ReconLoiter);
            }
            _ => panic!("expected recon ao"),
        }
        match &intents[1] {
            MissionIntent::UpsertDroneAo(ao) => {
                assert_eq!(ao.mission, DroneMissionKind::Strike);
                assert_eq!(ao.id, 11);
            }
            _ => panic!("expected strike ao"),
        }
        let mut q = MissionIntentQueue::default();
        for i in intents {
            q.enqueue(i).unwrap();
        }
        assert_eq!(q.len(), 2);
    }

}
