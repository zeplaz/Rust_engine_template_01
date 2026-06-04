//! AC-003 — hybrid bridge between clipmap L1 and legacy [`AtmosphereField`].

use bevy::prelude::*;

use super::{AtmosphereClipmapStack};

/// Copy legacy field smoke/fog into clipmap L1 (alias ingest).
pub fn sync_l1_from_legacy_field(stack: &mut AtmosphereClipmapStack, legacy: &crate::systems::atmosphere::AtmosphereField) {
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

/// Push clipmap L1 back into legacy field (alias egress).
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
