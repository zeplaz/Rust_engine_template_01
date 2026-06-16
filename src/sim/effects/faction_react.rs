//! FACTION-REACT-001 — read-only telemetry vocabulary for faction stress hooks.
//!
//! Producers write via drain only; strategic consumers scan [`SimEffectTelemetryLedger`] rows.

use bevy::prelude::*;

use super::event::SimEffectSource;
use super::telemetry::{SimEffectTelemetryLedger, SimEffectTelemetryRecord};

/// Matches [`super::event::SimEffectKind::StructureHeat`] dedupe tag.
pub const STRUCTURE_HEAT_KIND_TAG: u8 = 4;

/// Matches [`super::event::SimEffectKind::LandscapeDisturbance`] dedupe tag (FACTION-REACT-002).
pub const LANDSCAPE_DISTURBANCE_KIND_TAG: u8 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactionStressTelemetryClass {
    /// Structure heat / fire catastrophe telemetry row.
    StructureFire,
    /// Grid overload → structure heat (power loss edge).
    PowerLoss,
    /// Landscape harvest / clearance disturbance (ecological stress).
    EcologicalDisturbance,
}

#[derive(Clone, Copy, Debug)]
pub struct FactionStressHook {
    pub effect_id: u64,
    pub class: FactionStressTelemetryClass,
    pub severity: f32,
}

/// Witness for lib + live proof — updated by strategic read-only consumer only.
#[derive(Resource, Debug, Default, Clone)]
pub struct SimEffectFactionReactWitness {
    pub wired: bool,
    pub hook_rows: u64,
    scan_cursor: u64,
}

impl SimEffectFactionReactWitness {
    pub fn scan_cursor(&self) -> u64 {
        self.scan_cursor
    }

    pub fn advance_cursor(&mut self, effect_id: u64) {
        self.scan_cursor = self.scan_cursor.max(effect_id);
    }

    pub fn record_hooks(&mut self, count: u64) {
        self.wired = true;
        self.hook_rows = self.hook_rows.saturating_add(count);
    }
}

#[must_use]
pub fn classify_faction_stress_row(row: &SimEffectTelemetryRecord) -> Option<FactionStressTelemetryClass> {
    if !row.dispatch_ok {
        return None;
    }
    if row.kind_tag == LANDSCAPE_DISTURBANCE_KIND_TAG {
        return Some(FactionStressTelemetryClass::EcologicalDisturbance);
    }
    if row.kind_tag != STRUCTURE_HEAT_KIND_TAG {
        return None;
    }
    Some(if row.source == SimEffectSource::GridOverload {
        FactionStressTelemetryClass::PowerLoss
    } else {
        FactionStressTelemetryClass::StructureFire
    })
}

#[must_use]
pub fn stress_severity(class: FactionStressTelemetryClass) -> f32 {
    match class {
        FactionStressTelemetryClass::StructureFire => 0.08,
        FactionStressTelemetryClass::PowerLoss => 0.12,
        FactionStressTelemetryClass::EcologicalDisturbance => 0.05,
    }
}

/// Scan ledger rows after `cursor`; advances cursor to newest seen `effect_id`.
#[must_use]
pub fn scan_faction_stress_rows(
    ledger: &SimEffectTelemetryLedger,
    cursor: u64,
) -> (Vec<FactionStressHook>, u64) {
    let mut hooks = Vec::new();
    let mut max_id = cursor;
    for row in ledger.rows.iter().filter(|r| r.effect_id > cursor) {
        max_id = max_id.max(row.effect_id);
        if let Some(class) = classify_faction_stress_row(row) {
            hooks.push(FactionStressHook {
                effect_id: row.effect_id,
                class,
                severity: stress_severity(class),
            });
        }
    }
    (hooks, max_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::effects::event::{SimEffectEvent, SimEffectKind, SimEffectSource};

    #[test]
    fn classifies_structure_heat_and_grid_overload_rows() {
        let mut ledger = SimEffectTelemetryLedger::default();
        ledger.record_drain(
            1,
            &SimEffectEvent {
                source: SimEffectSource::GridOverload,
                cause_id: "CAUSE-grid-1".into(),
                parent_effect_id: None,
                kind: SimEffectKind::StructureHeat {
                    chunk: IVec2::ZERO,
                    cells: vec![(0, 0.3)],
                },
            },
            true,
        );
        ledger.record_drain(
            2,
            &SimEffectEvent {
                source: SimEffectSource::ScenarioScript,
                cause_id: "CAUSE-fire-1".into(),
                parent_effect_id: None,
                kind: SimEffectKind::StructureHeat {
                    chunk: IVec2::new(1, 0),
                    cells: vec![(1, 0.5)],
                },
            },
            true,
        );
        let (hooks, max_id) = scan_faction_stress_rows(&ledger, 0);
        assert_eq!(hooks.len(), 2);
        assert!(max_id >= 2);
        assert!(hooks.iter().any(|h| h.class == FactionStressTelemetryClass::PowerLoss));
        assert!(hooks.iter().any(|h| h.class == FactionStressTelemetryClass::StructureFire));
    }
}
