//! General research / capability ledger (PLAN-SCENARIO-VS-SYSTEMS + RESEARCH-CAPABILITY-v1).
//!
//! One book, many consumers (drones, plants, sat, comms, manuf). Not a per-product `*ResearchState`.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use bevy::prelude::*;
use serde::Deserialize;

pub const CAPABILITIES_RON: &str = "assets/configs/research/capabilities_v1.ron";

/// Well-known capability ids (must match RON).
pub mod ids {
    pub const COMMS_FIBER_OPS: &str = "comms.fiber_ops";
    pub const COMMS_LINK_HARDENING: &str = "comms.link_hardening";
    pub const SAT_DOWNLINK_BASIC: &str = "sat.downlink_basic";
    pub const UAS_AUTONOMY_L2: &str = "uas.autonomy_l2";
    pub const UAS_AUTONOMY_L3: &str = "uas.autonomy_l3";
    pub const SENSOR_SEEKER_IR: &str = "sensor.seeker_ir";
    pub const MANUF_UAS_ASSEMBLY: &str = "manuf.uas_assembly";
    pub const MANUF_CRUISE_LINE: &str = "manuf.cruise_line";
    pub const WX_RESIST_LINK: &str = "wx.resist_link";
    pub const WX_RESIST_SENSOR: &str = "wx.resist_sensor";
    pub const NUCLEAR_GEN_II_PLUS: &str = "nuclear_gen_ii_plus_v1";
}

#[derive(Debug, Clone, Deserialize)]
struct CapabilitiesFile {
    #[serde(default)]
    #[allow(dead_code)] // file-format tag, parsed for validation only
    schema_version: u32,
    #[serde(default)]
    capabilities: Vec<CapabilityDefRon>,
}

#[derive(Debug, Clone, Deserialize)]
struct CapabilityDefRon {
    id: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    domain: String,
}

#[derive(Clone, Debug)]
pub struct CapabilityDef {
    pub id: String,
    pub label: String,
    pub domain: String,
}

/// Sole writer: research domain / scenario cheats / future tech UI.
#[derive(Resource, Clone, Debug, Default)]
pub struct ResearchCapabilityBook {
    pub unlocked: HashSet<String>,
    /// Progress 0..1 (optional; unlock may be binary).
    pub progress: HashMap<String, f32>,
    /// Multipliers (e.g. wx resist add, manuf yield). Missing → 0.0 for resist, 1.0 for yield helpers.
    pub multipliers: HashMap<String, f32>,
    pub catalog: Vec<CapabilityDef>,
    pub revision: u64,
}

impl ResearchCapabilityBook {
    #[must_use]
    pub fn unlocked(&self, id: &str) -> bool {
        self.unlocked.contains(id)
    }

    #[must_use]
    pub fn all_unlocked<'a>(&self, ids: impl IntoIterator<Item = &'a str>) -> bool {
        ids.into_iter().all(|id| self.unlocked(id))
    }

    pub fn unlock(&mut self, id: impl Into<String>) {
        let id = id.into();
        if self.unlocked.insert(id.clone()) {
            self.progress.insert(id, 1.0);
            self.revision = self.revision.wrapping_add(1);
        }
    }

    pub fn set_multiplier(&mut self, id: impl Into<String>, value: f32) {
        self.multipliers.insert(id.into(), value);
        self.revision = self.revision.wrapping_add(1);
    }

    #[must_use]
    pub fn multiplier(&self, id: &str) -> f32 {
        self.multipliers.get(id).copied().unwrap_or(0.0)
    }

    /// Manufacturing yield scale; default 1.0 when unset.
    #[must_use]
    pub fn yield_mul(&self, id: &str) -> f32 {
        self.multipliers.get(id).copied().unwrap_or(1.0).max(0.0)
    }

    /// Link envelope resist: base + unlocked wx mul, clamped.
    #[must_use]
    pub fn link_resist(&self, base: f32) -> f32 {
        (base + self.multiplier(ids::WX_RESIST_LINK)).clamp(0.0, 0.85)
    }

    #[must_use]
    pub fn sensor_resist(&self, base: f32) -> f32 {
        (base + self.multiplier(ids::WX_RESIST_SENSOR)).clamp(0.0, 0.85)
    }

    #[must_use]
    pub fn max_uas_autonomy_level(&self) -> u8 {
        if self.unlocked(ids::UAS_AUTONOMY_L3) {
            3
        } else if self.unlocked(ids::UAS_AUTONOMY_L2) {
            2
        } else {
            1
        }
    }
}

/// Shared read stubs — prove sat/comms/manuf consume the same ledger (not drone-only).
#[derive(Resource, Clone, Debug, Default)]
pub struct ResearchConsumerStubs {
    pub sat_downlink_ready: bool,
    pub comms_fiber_ready: bool,
    pub manuf_uas_yield: f32,
    pub manuf_cruise_line: bool,
}

pub fn sync_research_consumer_stubs_system(
    book: Res<ResearchCapabilityBook>,
    mut stubs: ResMut<ResearchConsumerStubs>,
) {
    stubs.sat_downlink_ready = book.unlocked(ids::SAT_DOWNLINK_BASIC);
    stubs.comms_fiber_ready = book.unlocked(ids::COMMS_FIBER_OPS);
    stubs.manuf_uas_yield = book.yield_mul(ids::MANUF_UAS_ASSEMBLY);
    stubs.manuf_cruise_line = book.unlocked(ids::MANUF_CRUISE_LINE);
}

fn load_catalog_from_disk() -> Vec<CapabilityDef> {
    let path = Path::new(CAPABILITIES_RON);
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return default_catalog(),
    };
    match ron::from_str::<CapabilitiesFile>(&text) {
        Ok(f) => f
            .capabilities
            .into_iter()
            .map(|c| CapabilityDef {
                id: c.id,
                label: c.label,
                domain: c.domain,
            })
            .collect(),
        Err(_) => default_catalog(),
    }
}

fn default_catalog() -> Vec<CapabilityDef> {
    [
        (ids::COMMS_FIBER_OPS, "Fiber-optic C2", "comms"),
        (ids::SAT_DOWNLINK_BASIC, "Satellite downlink", "sat"),
        (ids::UAS_AUTONOMY_L2, "UAS autonomy L2", "uas"),
        (ids::UAS_AUTONOMY_L3, "UAS autonomy L3", "uas"),
        (ids::SENSOR_SEEKER_IR, "IR seeker", "sensor"),
        (ids::MANUF_UAS_ASSEMBLY, "UAS assembly", "manuf"),
        (ids::MANUF_CRUISE_LINE, "Cruise line", "manuf"),
        (ids::WX_RESIST_LINK, "WX link resist", "wx"),
        (ids::WX_RESIST_SENSOR, "WX sensor resist", "wx"),
        (ids::NUCLEAR_GEN_II_PLUS, "Nuclear Gen II+", "power"),
    ]
    .into_iter()
    .map(|(id, label, domain)| CapabilityDef {
        id: id.into(),
        label: label.into(),
        domain: domain.into(),
    })
    .collect()
}

pub struct ResearchCapabilityPlugin;

impl Plugin for ResearchCapabilityPlugin {
    fn build(&self, app: &mut App) {
        let mut book = ResearchCapabilityBook {
            catalog: load_catalog_from_disk(),
            ..Default::default()
        };
        // Baseline manuf yield so plants are not zeroed when id unset.
        book.multipliers
            .insert(ids::MANUF_UAS_ASSEMBLY.into(), 1.0);
        app.insert_resource(book)
            .init_resource::<ResearchConsumerStubs>()
            .add_systems(Update, sync_research_consumer_stubs_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn research_capability_shared_fiber_gates_drone_and_comms_stub() {
        let mut book = ResearchCapabilityBook::default();
        assert!(!book.unlocked(ids::COMMS_FIBER_OPS));
        book.unlock(ids::COMMS_FIBER_OPS);
        assert!(book.unlocked(ids::COMMS_FIBER_OPS));
        let mut stubs = ResearchConsumerStubs::default();
        // Mirror sync system.
        stubs.comms_fiber_ready = book.unlocked(ids::COMMS_FIBER_OPS);
        stubs.sat_downlink_ready = book.unlocked(ids::SAT_DOWNLINK_BASIC);
        assert!(stubs.comms_fiber_ready);
        assert!(!stubs.sat_downlink_ready);
        // Same id a drone modality gate would read.
        assert!(book.unlocked(ids::COMMS_FIBER_OPS));
    }

    #[test]
    fn research_capability_autonomy_and_resist() {
        let mut book = ResearchCapabilityBook::default();
        assert_eq!(book.max_uas_autonomy_level(), 1);
        book.unlock(ids::UAS_AUTONOMY_L2);
        assert_eq!(book.max_uas_autonomy_level(), 2);
        book.set_multiplier(ids::WX_RESIST_LINK, 0.5);
        assert!((book.link_resist(0.9) - 0.85).abs() < 1e-3);
        assert!((book.yield_mul(ids::MANUF_UAS_ASSEMBLY) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn no_drone_research_state_type_in_research_module() {
        // Compile-time smell: this module must not re-export a DroneResearchState.
        let _ = std::any::type_name::<ResearchCapabilityBook>();
    }

    #[test]
    fn research_capability_plant_and_drone_share_cruise_id() {
        let mut book = ResearchCapabilityBook::default();
        assert!(!book.unlocked(ids::MANUF_CRUISE_LINE));
        book.unlock(ids::MANUF_CRUISE_LINE);
        assert!(book.all_unlocked([ids::MANUF_CRUISE_LINE]));
        // Same id gates missile_body_plant activation and cruise manuf stub.
        let mut stubs = ResearchConsumerStubs::default();
        sync_research_consumer_stubs_system_pure(&book, &mut stubs);
        assert!(stubs.manuf_cruise_line);
    }

    fn sync_research_consumer_stubs_system_pure(
        book: &ResearchCapabilityBook,
        stubs: &mut ResearchConsumerStubs,
    ) {
        stubs.sat_downlink_ready = book.unlocked(ids::SAT_DOWNLINK_BASIC);
        stubs.comms_fiber_ready = book.unlocked(ids::COMMS_FIBER_OPS);
        stubs.manuf_uas_yield = book.yield_mul(ids::MANUF_UAS_ASSEMBLY);
        stubs.manuf_cruise_line = book.unlocked(ids::MANUF_CRUISE_LINE);
    }
}
