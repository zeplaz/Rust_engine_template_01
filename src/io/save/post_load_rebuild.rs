//! IO-07 / IO-08 — rebuild derived save DTO fields on hydrate, never on wire (E7).
//!
//! See `src/dev/plan_ref_books_upgrade_v2.md` slice E7 for the field audit.

use super::fire_overlay::SavedFireChunkRow;
use super::stand_overlay::SavedCellStandRow;

/// `burnt_count` is derived from per-cell `burnt_at` (ticks since burn; negative = never).
#[must_use]
pub fn burnt_count_from_burnt_at(burnt_at: &[f32]) -> u32 {
    burnt_at
        .iter()
        .filter(|t| t.is_finite() && **t >= 0.0)
        .count() as u32
}

/// Rebuild stand-row derived fields after load (IO-08).
pub fn rebuild_saved_cell_stand_row_derived(row: &mut SavedCellStandRow) {
    row.burnt_count = burnt_count_from_burnt_at(&row.burnt_at);
}

/// Strip derived ecology/fire vectors before writing a new overlay artifact.
pub fn strip_stand_row_derived_for_save(row: &mut SavedCellStandRow) {
    row.burnt_count = 0;
    row.heat.clear();
    row.fuel.clear();
    row.burn_hold.clear();
}

/// Rebuild fire-row derived flags after load (IO-08).
///
/// `has_overlay` follows the per-cell vectors. Smoke, toxic, and cell
/// heat/fuel/burn_hold are persisted (schema v2) and must not be cleared here.
pub fn rebuild_saved_fire_chunk_row_derived(row: &mut SavedFireChunkRow) {
    row.has_overlay = !row.cell_heat.is_empty()
        && row.cell_heat.len() == row.cell_fuel.len()
        && row.cell_heat.len() == row.cell_burn_hold.len();
}

/// Omit the derived `has_overlay` flag from new saves. Cell fire and chunk
/// smoke/toxic stay on the wire.
pub fn strip_fire_row_derived_for_save(row: &mut SavedFireChunkRow) {
    row.has_overlay = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn burnt_count_rebuilt_from_burnt_at_clock() {
        let mut row = SavedCellStandRow {
            chunk: [0, 0],
            kind: vec![1; 4],
            full_cap: vec![0.5; 4],
            fuel_cap: vec![0.0; 4],
            burnt_at: vec![12.0, -1.0, 40.0, -1.0],
            burnt_count: 99,
            seeded: true,
            heat: vec![0.1; 4],
            fuel: vec![],
            burn_hold: vec![],
        };
        rebuild_saved_cell_stand_row_derived(&mut row);
        assert_eq!(row.burnt_count, 2);
    }

    #[test]
    fn strip_stand_row_clears_derived_fire_vectors() {
        let mut row = SavedCellStandRow {
            chunk: [0, 0],
            kind: vec![1],
            full_cap: vec![1.0],
            fuel_cap: vec![0.0],
            burnt_at: vec![-1.0],
            burnt_count: 1,
            seeded: false,
            heat: vec![0.5],
            fuel: vec![0.2],
            burn_hold: vec![3],
        };
        strip_stand_row_derived_for_save(&mut row);
        assert!(row.heat.is_empty());
        assert!(row.fuel.is_empty());
        assert!(row.burn_hold.is_empty());
        assert_eq!(row.burnt_count, 0);
    }
}