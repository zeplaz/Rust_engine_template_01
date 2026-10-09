//! **Front / combat balance table** (PLAN-FRONT-COMBAT-v1 FC-1).
//!
//! One RON table drives every tunable in [`crate::combat`]. The compiled-in copy of
//! `assets/configs/combat/front_balance_v1.ron` is the default; [`load_front_balance`] re-reads the
//! file at runtime so designers can tune without rebuilding. Metric units throughout.

use std::path::{Path, PathBuf};

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Repo-relative path of the balance table.
pub const FRONT_BALANCE_RON: &str = "assets/configs/combat/front_balance_v1.ron";

const EMBEDDED_FRONT_BALANCE: &str = include_str!("../../assets/configs/combat/front_balance_v1.ron");

/// Formation role. Order is the stable index used by symbology and witness output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UnitKind {
    Infantry,
    Mechanized,
    Armor,
    Artillery,
    Recon,
    Engineer,
    AirDefense,
}

impl UnitKind {
    pub const ALL: [UnitKind; 7] = [
        UnitKind::Infantry,
        UnitKind::Mechanized,
        UnitKind::Armor,
        UnitKind::Artillery,
        UnitKind::Recon,
        UnitKind::Engineer,
        UnitKind::AirDefense,
    ];

    /// Vehicles that anti-armor fortification protects against.
    #[must_use]
    pub const fn is_armored(self) -> bool {
        matches!(self, UnitKind::Armor | UnitKind::Mechanized)
    }
}

/// Fortification family. Maps from defense `SiteArchetype` in `combat::fortification`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FortKind {
    Trench,
    Bunker,
    Wall,
    DragonTeeth,
    Minefield,
    Wire,
}

impl FortKind {
    pub const ALL: [FortKind; 6] = [
        FortKind::Trench,
        FortKind::Bunker,
        FortKind::Wall,
        FortKind::DragonTeeth,
        FortKind::Minefield,
        FortKind::Wire,
    ];
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrontThresholds {
    pub own_min: f32,
    pub contest_band: f32,
    pub derive_every_ticks: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlParams {
    pub half_life_s: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TheaterParams {
    pub max_len_m: f32,
    pub depth_m: f32,
    pub id_keep_overlap: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PressureWeights {
    pub w_threat: f32,
    pub w_artillery: f32,
    pub w_contest: f32,
    pub w_supply: f32,
    pub w_mobility: f32,
    pub w_fortification: f32,
    pub nms_radius_m: f32,
    pub top_k: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CombatParams {
    pub step_s: f32,
    pub k_attrition: f32,
    pub morale_loss_per_strength: f32,
    pub morale_recovery_per_s: f32,
    pub break_threshold: f32,
    pub rally_threshold: f32,
    pub supply_burn_per_s: f32,
    pub min_supply_factor: f32,
    pub min_morale_factor: f32,
    pub attack_mult: f32,
    pub defend_mult: f32,
    pub retreat_mult: f32,
    pub attacker_cover_scale: f32,
}

/// Sustainment draw in kilograms per second at full strength, combat posture.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DrawRates {
    pub ammunition: f32,
    pub fuel: f32,
    pub rations: f32,
    pub engineering: f32,
}

impl DrawRates {
    #[must_use]
    pub fn total(&self) -> f32 {
        self.ammunition + self.fuel + self.rations + self.engineering
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitProfile {
    pub kind: UnitKind,
    pub firepower: f32,
    pub range_m: f32,
    pub control_radius_m: f32,
    pub speed_mps: f32,
    pub toe_personnel: u32,
    pub draw_kg_per_s: DrawRates,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EffectivenessRow {
    pub attacker: UnitKind,
    pub target: UnitKind,
    pub mult: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FortificationProfile {
    pub kind: FortKind,
    pub cover: f32,
    pub obstacle_slow: f32,
    pub mobility_cost_add: f32,
    pub anti_armor: f32,
    pub build_effort_person_hours: f32,
}

/// The whole table. Inserted as a resource by `CombatPlugin`.
#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrontBalance {
    pub schema_version: u32,
    pub table_id: String,
    pub tile_size_m: f32,
    pub front: FrontThresholds,
    pub control: ControlParams,
    pub theater: TheaterParams,
    pub pressure: PressureWeights,
    pub combat: CombatParams,
    pub units: Vec<UnitProfile>,
    pub effectiveness_default: f32,
    pub effectiveness: Vec<EffectivenessRow>,
    pub fortifications: Vec<FortificationProfile>,
}

impl Default for FrontBalance {
    fn default() -> Self {
        parse_front_balance(EMBEDDED_FRONT_BALANCE)
            .expect("embedded front_balance_v1.ron must parse (combat::balance tests guard this)")
    }
}

impl FrontBalance {
    /// Profile for a unit kind. Validation guarantees every kind has one.
    #[must_use]
    pub fn unit(&self, kind: UnitKind) -> &UnitProfile {
        self.units
            .iter()
            .find(|u| u.kind == kind)
            .unwrap_or(&self.units[0])
    }

    /// Attacker → target damage multiplier.
    #[must_use]
    pub fn effectiveness(&self, attacker: UnitKind, target: UnitKind) -> f32 {
        self.effectiveness
            .iter()
            .find(|r| r.attacker == attacker && r.target == target)
            .map_or(self.effectiveness_default, |r| r.mult)
    }

    #[must_use]
    pub fn fortification(&self, kind: FortKind) -> Option<&FortificationProfile> {
        self.fortifications.iter().find(|f| f.kind == kind)
    }

    /// Metres → whole tiles (at least one).
    #[must_use]
    pub fn metres_to_tiles(&self, metres: f32) -> f32 {
        (metres / self.tile_size_m.max(f32::EPSILON)).max(0.0)
    }

    /// Per-second exponential decay rate from the control half-life.
    #[must_use]
    pub fn control_decay_rate(&self) -> f32 {
        if self.control.half_life_s <= 0.0 {
            return 0.0;
        }
        std::f32::consts::LN_2 / self.control.half_life_s
    }

    /// Every rule a designer edit can break. Empty = valid.
    #[must_use]
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let unit01 = |name: &str, v: f32, issues: &mut Vec<String>| {
            if !(0.0..=1.0).contains(&v) || !v.is_finite() {
                issues.push(format!("{name} must be in 0..=1, got {v}"));
            }
        };
        let positive = |name: &str, v: f32, issues: &mut Vec<String>| {
            if v <= 0.0 || !v.is_finite() {
                issues.push(format!("{name} must be > 0, got {v}"));
            }
        };
        let non_negative = |name: &str, v: f32, issues: &mut Vec<String>| {
            if v < 0.0 || !v.is_finite() {
                issues.push(format!("{name} must be >= 0, got {v}"));
            }
        };
        if self.schema_version != 1 {
            issues.push(format!("schema_version {} unsupported (want 1)", self.schema_version));
        }
        positive("tile_size_m", self.tile_size_m, &mut issues);
        unit01("front.own_min", self.front.own_min, &mut issues);
        unit01("front.contest_band", self.front.contest_band, &mut issues);
        if self.front.derive_every_ticks == 0 {
            issues.push("front.derive_every_ticks must be >= 1".into());
        }
        non_negative("control.half_life_s", self.control.half_life_s, &mut issues);
        positive("theater.max_len_m", self.theater.max_len_m, &mut issues);
        positive("theater.depth_m", self.theater.depth_m, &mut issues);
        unit01("theater.id_keep_overlap", self.theater.id_keep_overlap, &mut issues);
        let p = &self.pressure;
        for (n, v) in [
            ("pressure.w_threat", p.w_threat),
            ("pressure.w_artillery", p.w_artillery),
            ("pressure.w_contest", p.w_contest),
            ("pressure.w_supply", p.w_supply),
            ("pressure.w_mobility", p.w_mobility),
            ("pressure.w_fortification", p.w_fortification),
            ("pressure.nms_radius_m", p.nms_radius_m),
        ] {
            non_negative(n, v, &mut issues);
        }
        if p.top_k == 0 {
            issues.push("pressure.top_k must be >= 1".into());
        }
        let c = &self.combat;
        positive("combat.step_s", c.step_s, &mut issues);
        positive("combat.k_attrition", c.k_attrition, &mut issues);
        non_negative("combat.morale_loss_per_strength", c.morale_loss_per_strength, &mut issues);
        non_negative("combat.morale_recovery_per_s", c.morale_recovery_per_s, &mut issues);
        unit01("combat.break_threshold", c.break_threshold, &mut issues);
        unit01("combat.rally_threshold", c.rally_threshold, &mut issues);
        if c.rally_threshold <= c.break_threshold {
            issues.push("combat.rally_threshold must exceed break_threshold (hysteresis)".into());
        }
        non_negative("combat.supply_burn_per_s", c.supply_burn_per_s, &mut issues);
        unit01("combat.min_supply_factor", c.min_supply_factor, &mut issues);
        unit01("combat.min_morale_factor", c.min_morale_factor, &mut issues);
        for (n, v) in [
            ("combat.attack_mult", c.attack_mult),
            ("combat.defend_mult", c.defend_mult),
            ("combat.retreat_mult", c.retreat_mult),
        ] {
            non_negative(n, v, &mut issues);
        }
        unit01("combat.attacker_cover_scale", c.attacker_cover_scale, &mut issues);
        for kind in UnitKind::ALL {
            let rows: Vec<_> = self.units.iter().filter(|u| u.kind == kind).collect();
            match rows.len() {
                0 => issues.push(format!("units: missing profile for {kind:?}")),
                1 => {
                    let u = rows[0];
                    positive(&format!("units[{kind:?}].firepower"), u.firepower, &mut issues);
                    positive(&format!("units[{kind:?}].range_m"), u.range_m, &mut issues);
                    positive(&format!("units[{kind:?}].control_radius_m"), u.control_radius_m, &mut issues);
                    positive(&format!("units[{kind:?}].speed_mps"), u.speed_mps, &mut issues);
                    if u.toe_personnel == 0 {
                        issues.push(format!("units[{kind:?}].toe_personnel must be >= 1"));
                    }
                    let d = u.draw_kg_per_s;
                    for (n, v) in [
                        ("ammunition", d.ammunition),
                        ("fuel", d.fuel),
                        ("rations", d.rations),
                        ("engineering", d.engineering),
                    ] {
                        non_negative(&format!("units[{kind:?}].draw_kg_per_s.{n}"), v, &mut issues);
                    }
                }
                n => issues.push(format!("units: {n} profiles for {kind:?} (want 1)")),
            }
        }
        non_negative("effectiveness_default", self.effectiveness_default, &mut issues);
        let mut seen = std::collections::BTreeSet::new();
        for row in &self.effectiveness {
            if !seen.insert((row.attacker, row.target)) {
                issues.push(format!("effectiveness: duplicate row {:?}->{:?}", row.attacker, row.target));
            }
            non_negative(
                &format!("effectiveness[{:?}->{:?}]", row.attacker, row.target),
                row.mult,
                &mut issues,
            );
        }
        for kind in FortKind::ALL {
            let rows: Vec<_> = self.fortifications.iter().filter(|f| f.kind == kind).collect();
            match rows.len() {
                0 => issues.push(format!("fortifications: missing profile for {kind:?}")),
                1 => {
                    let f = rows[0];
                    unit01(&format!("fortifications[{kind:?}].cover"), f.cover, &mut issues);
                    unit01(&format!("fortifications[{kind:?}].obstacle_slow"), f.obstacle_slow, &mut issues);
                    unit01(&format!("fortifications[{kind:?}].mobility_cost_add"), f.mobility_cost_add, &mut issues);
                    unit01(&format!("fortifications[{kind:?}].anti_armor"), f.anti_armor, &mut issues);
                    if f.cover + f.anti_armor > 0.95 {
                        issues.push(format!(
                            "fortifications[{kind:?}]: cover + anti_armor must stay <= 0.95 (no invulnerable tiles)"
                        ));
                    }
                    positive(
                        &format!("fortifications[{kind:?}].build_effort_person_hours"),
                        f.build_effort_person_hours,
                        &mut issues,
                    );
                }
                n => issues.push(format!("fortifications: {n} profiles for {kind:?} (want 1)")),
            }
        }
        issues
    }
}

/// Parse a balance table from RON text.
pub fn parse_front_balance(text: &str) -> Result<FrontBalance, String> {
    ron::from_str(text).map_err(|e| format!("front balance RON parse: {e}"))
}

#[must_use]
fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

pub fn load_front_balance_from_path(path: &Path) -> Result<FrontBalance, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let table = parse_front_balance(&text)?;
    let issues = table.validate();
    if issues.is_empty() {
        Ok(table)
    } else {
        Err(format!("{}: {}", path.display(), issues.join("; ")))
    }
}

/// Runtime load: disk table when present and valid, else the compiled-in default (with the error).
#[must_use]
pub fn load_front_balance() -> (FrontBalance, Option<String>) {
    match load_front_balance_from_path(&repo_asset_path(FRONT_BALANCE_RON)) {
        Ok(table) => (table, None),
        Err(err) => (FrontBalance::default(), Some(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_table_parses_and_validates() {
        let table = FrontBalance::default();
        assert_eq!(table.table_id, "front_balance_v1");
        assert_eq!(table.validate(), Vec::<String>::new());
    }

    #[test]
    fn disk_table_matches_embedded() {
        let (table, err) = load_front_balance();
        assert!(err.is_none(), "{err:?}");
        assert_eq!(table, FrontBalance::default());
    }

    #[test]
    fn validation_catches_broken_edits() {
        let mut t = FrontBalance::default();
        t.front.own_min = 1.5;
        t.units.retain(|u| u.kind != UnitKind::Recon);
        t.combat.rally_threshold = t.combat.break_threshold;
        t.fortifications[0].cover = 0.9;
        t.fortifications[0].anti_armor = 0.5;
        let issues = t.validate();
        assert!(issues.iter().any(|i| i.contains("own_min")));
        assert!(issues.iter().any(|i| i.contains("Recon")));
        assert!(issues.iter().any(|i| i.contains("hysteresis")));
        assert!(issues.iter().any(|i| i.contains("invulnerable")));
    }

    #[test]
    fn effectiveness_lookup_and_default() {
        let t = FrontBalance::default();
        assert!(t.effectiveness(UnitKind::Armor, UnitKind::Infantry) > 1.0);
        assert!(t.effectiveness(UnitKind::Infantry, UnitKind::Armor) < 1.0);
        assert_eq!(
            t.effectiveness(UnitKind::Engineer, UnitKind::Recon),
            t.effectiveness_default
        );
    }

    /// Balance invariants designers rely on (rock-paper-scissors shape, fortification ordering).
    #[test]
    fn balance_shape_invariants() {
        let t = FrontBalance::default();
        let fp = |k| t.unit(k).firepower;
        // Armor out-hits infantry head-on, but a bunker flips the trade.
        let armor_vs_inf = fp(UnitKind::Armor) * t.effectiveness(UnitKind::Armor, UnitKind::Infantry);
        let inf_vs_armor = fp(UnitKind::Infantry) * t.effectiveness(UnitKind::Infantry, UnitKind::Armor);
        assert!(armor_vs_inf > inf_vs_armor);
        let bunker = t.fortification(FortKind::Bunker).unwrap();
        let trench = t.fortification(FortKind::Trench).unwrap();
        assert!(bunker.cover > trench.cover);
        assert!(bunker.build_effort_person_hours > trench.build_effort_person_hours);
        // Artillery outranges everything; recon is fastest.
        for k in UnitKind::ALL {
            assert!(t.unit(UnitKind::Artillery).range_m >= t.unit(k).range_m);
            assert!(t.unit(UnitKind::Recon).speed_mps >= t.unit(k).speed_mps);
        }
        // Obstacles slow more than they cover.
        for k in [FortKind::DragonTeeth, FortKind::Minefield, FortKind::Wire] {
            let f = t.fortification(k).unwrap();
            assert!(f.obstacle_slow > f.cover, "{k:?}");
        }
    }

    #[test]
    fn half_life_decay_rate() {
        let t = FrontBalance::default();
        let r = t.control_decay_rate();
        let after = (-r * t.control.half_life_s).exp();
        assert!((after - 0.5).abs() < 1e-4);
    }
}
