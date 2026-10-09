//! **Combat / front domain** (fleet `front`, PLAN-FRONT-COMBAT-v1 — `src/dev/plan_front_combat_v1.md`).
//!
//! Pure, headless, deterministic cores live here; Bevy systems are thin wrappers.
//! Attaches to authoritative fields: `ChunkStrategicOverlay` (control / threat), `FrontlineState`,
//! construction sites (fortification), the `Zone` pass (formation presence). No second field store.

use bevy::prelude::*;

pub mod balance;
pub mod front;

pub use balance::{
    load_front_balance, load_front_balance_from_path, parse_front_balance, FortKind,
    FortificationProfile, FrontBalance, UnitKind, UnitProfile, FRONT_BALANCE_RON,
};
pub use front::{
    classify_cell, derive_fronts, ControlGrid, FrontCell, FrontDerivation, FrontPolyline, FrontSegment,
    Hostility,
};

/// Load outcome of [`FrontBalance`] (disk error keeps the compiled-in default).
#[derive(Resource, Clone, Debug, Default)]
pub struct FrontBalanceLoadReport {
    pub error: Option<String>,
}

/// Registers front / combat resources and systems. Added by [`crate::strategic::FactionPlugin`].
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<FrontBalance>() {
            let (balance, error) = load_front_balance();
            if let Some(err) = &error {
                warn!("front balance: using compiled-in default ({err})");
            }
            app.insert_resource(balance)
                .insert_resource(FrontBalanceLoadReport { error });
        }
    }
}
