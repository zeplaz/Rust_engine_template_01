//! DEBT-006 — opt-in hybrid bridge between clipmap L1 and legacy [`AtmosphereField`].
//!
//! **Default OFF** (ES-6): clipmap L0 is sim authority for smoke consumers that already
//! read [`AtmosphereClipmapStack`]. Set `RUST_ENGINE_ATMOS_LEGACY_BRIDGE=1` to re-enable
//! the L1↔128² roundtrip for rollback only. Do not treat this bridge as dual authority.

use bevy::prelude::*;

use super::AtmosphereClipmapStack;

/// Env rollback — legacy L1↔field sync runs only when set to `1` / `true` / `on`.
#[must_use]
pub fn legacy_atmosphere_bridge_enabled() -> bool {
    matches!(
        std::env::var("RUST_ENGINE_ATMOS_LEGACY_BRIDGE").as_deref(),
        Ok("1") | Ok("true") | Ok("on")
    )
}

/// Copy legacy field smoke/fog into clipmap L1 (alias ingest) — rollback path only.
pub fn sync_l1_from_legacy_field(
    stack: &mut AtmosphereClipmapStack,
    legacy: &crate::systems::atmosphere::AtmosphereField,
) {
    let Some(l1) = stack.levels.get_mut(1) else {
        return;
    };
    let legacy_smoke = legacy
        .cells
        .first()
        .map(|c| c.smoke_density)
        .unwrap_or(0.0);
    let legacy_fog = legacy
        .cells
        .first()
        .map(|c| c.fog_density)
        .unwrap_or(0.0);
    if let Some(cell) = l1.smoke_density.first_mut() {
        *cell = cell.max(legacy_smoke).max(legacy_fog * 0.5);
    }
}

/// Push clipmap L1 back into legacy field (alias egress) — rollback path only.
pub fn sync_legacy_field_from_l1(
    stack: &AtmosphereClipmapStack,
    legacy: &mut crate::systems::atmosphere::AtmosphereField,
) {
    let Some(l1) = stack.levels.get(1) else {
        return;
    };
    let l1_smoke = l1.smoke_density.first().copied().unwrap_or(0.0);
    if let Some(cell) = legacy.cells.first_mut() {
        cell.smoke_density = cell.smoke_density.max(l1_smoke);
        cell.fog_density = cell.fog_density.max(l1_smoke * 0.25);
    }
}

/// Scheduled only when [`legacy_atmosphere_bridge_enabled`] (env rollback).
pub fn legacy_atmosphere_bridge_system(
    mut stack: ResMut<AtmosphereClipmapStack>,
    legacy: Option<ResMut<crate::systems::atmosphere::AtmosphereField>>,
) {
    let Some(mut legacy) = legacy else {
        return;
    };
    sync_l1_from_legacy_field(stack.as_mut(), legacy.as_ref());
    sync_legacy_field_from_l1(stack.as_ref(), legacy.as_mut());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::atmosphere::AtmosphereField;

    #[test]
    fn legacy_bridge_gated_off_by_default() {
        let _ = std::env::remove_var("RUST_ENGINE_ATMOS_LEGACY_BRIDGE");
        assert!(!legacy_atmosphere_bridge_enabled());
    }

    #[test]
    fn legacy_field_l1_alias_roundtrip() {
        let mut stack = AtmosphereClipmapStack::default();
        let mut legacy = AtmosphereField::default();
        legacy.cells[0].smoke_density = 0.75;

        sync_l1_from_legacy_field(&mut stack, &legacy);
        stack.levels[1].smoke_density[0] = 0.5;
        sync_legacy_field_from_l1(&stack, &mut legacy);

        assert!(
            (legacy.cells[0].smoke_density - 0.75).abs() < 1e-4,
            "legacy smoke should roundtrip within epsilon"
        );
    }
}
