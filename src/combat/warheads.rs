//! DF-7 — warhead yield table (metres). Sole loader for combat blast radii / damage.

use bevy::prelude::*;
use serde::Deserialize;

pub const WARHEADS_RON: &str = "assets/configs/combat/warheads_v1.ron";

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct WarheadDef {
    pub id: String,
    pub mass_kg: f32,
    pub blast_radius_m: f32,
    pub peak_damage: f32,
    pub penetration_m: f32,
    pub cep_m: f32,
}

#[derive(Clone, Debug, Deserialize)]
struct WarheadsFile {
    warheads: Vec<WarheadDef>,
    default_id: String,
}

/// Loaded warhead catalog. Missing disk → compiled defaults.
#[derive(Resource, Clone, Debug)]
pub struct WarheadCatalog {
    pub warheads: Vec<WarheadDef>,
    pub default_id: String,
}

impl Default for WarheadCatalog {
    fn default() -> Self {
        Self {
            warheads: vec![
                WarheadDef {
                    id: "small".into(),
                    mass_kg: 250.0,
                    blast_radius_m: 24.0,
                    peak_damage: 55.0,
                    penetration_m: 2.0,
                    cep_m: 12.0,
                },
                WarheadDef {
                    id: "medium".into(),
                    mass_kg: 500.0,
                    blast_radius_m: 40.0,
                    peak_damage: 120.0,
                    penetration_m: 4.0,
                    cep_m: 18.0,
                },
                WarheadDef {
                    id: "large".into(),
                    mass_kg: 1000.0,
                    blast_radius_m: 64.0,
                    peak_damage: 220.0,
                    penetration_m: 8.0,
                    cep_m: 28.0,
                },
            ],
            default_id: "medium".into(),
        }
    }
}

impl WarheadCatalog {
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&WarheadDef> {
        self.warheads.iter().find(|w| w.id == id)
    }

    #[must_use]
    pub fn default_def(&self) -> &WarheadDef {
        self.get(&self.default_id)
            .or_else(|| self.warheads.first())
            .expect("warhead catalog non-empty")
    }
}

#[must_use]
pub fn parse_warheads(text: &str) -> Result<WarheadCatalog, String> {
    let file: WarheadsFile = ron::from_str(text).map_err(|e| e.to_string())?;
    if file.warheads.is_empty() {
        return Err("warheads list empty".into());
    }
    Ok(WarheadCatalog {
        warheads: file.warheads,
        default_id: file.default_id,
    })
}

#[must_use]
pub fn load_warheads() -> (WarheadCatalog, Option<String>) {
    match std::fs::read_to_string(WARHEADS_RON) {
        Ok(text) => match parse_warheads(&text) {
            Ok(c) => (c, None),
            Err(e) => (WarheadCatalog::default(), Some(e)),
        },
        Err(e) => (WarheadCatalog::default(), Some(e.to_string())),
    }
}

/// Player / scenario strike request (DF-7).
#[derive(Clone, Debug)]
pub struct StrikeOrder {
    pub origin_xz_m: Vec2,
    pub target_xz_m: Vec2,
    pub warhead_id: String,
}

#[derive(Resource, Default, Debug)]
pub struct StrikeOrderQueue {
    pub pending: Vec<StrikeOrder>,
}

impl StrikeOrderQueue {
    pub fn enqueue(&mut self, order: StrikeOrder) {
        self.pending.push(order);
    }
}

/// Drain strike orders into ballistic flights (metres throughout).
pub fn apply_strike_orders_system(
    mut queue: ResMut<StrikeOrderQueue>,
    catalog: Res<WarheadCatalog>,
    mut ballistics: Option<ResMut<super::ballistics::BallisticFlightBook>>,
    mut audio: Option<ResMut<crate::audio::AudioOneShotQueue>>,
) {
    if queue.pending.is_empty() {
        return;
    }
    let Some(book) = ballistics.as_mut() else {
        // Keep pending until BallisticFlightBook exists — do not drop strikes.
        return;
    };
    let orders = std::mem::take(&mut queue.pending);
    for order in orders {
        let wh = catalog
            .get(&order.warhead_id)
            .unwrap_or_else(|| catalog.default_def());
        if let Some(audio) = audio.as_mut() {
            audio.push_combat(
                crate::audio::AudioOneShotKind::MissileLaunch,
                order.origin_xz_m,
            );
        }
        // Speed ≈ 180 m/s → metres per 1 s sim step (clock may differ; field is m/step).
        book.launch_with_warhead(
            order.origin_xz_m,
            order.target_xz_m,
            180.0,
            wh,
            super::ballistics::BallisticKind::Missile,
            None,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warhead_yield_scales_radius() {
        let c = WarheadCatalog::default();
        let s = c.get("small").unwrap();
        let m = c.get("medium").unwrap();
        let l = c.get("large").unwrap();
        assert!(s.blast_radius_m < m.blast_radius_m);
        assert!(m.blast_radius_m < l.blast_radius_m);
        assert!(s.peak_damage < m.peak_damage);
        assert!(m.peak_damage < l.peak_damage);
    }

    #[test]
    fn parse_ron_file_shape() {
        let text = std::fs::read_to_string(WARHEADS_RON).expect("warheads on disk");
        let c = parse_warheads(&text).expect("parse");
        assert_eq!(c.default_id, "medium");
        assert_eq!(c.warheads.len(), 3);
    }
}
