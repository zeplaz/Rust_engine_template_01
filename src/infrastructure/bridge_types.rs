//! Bridge structure types (PLAN-ROADS-v1). A bridge is a structure, not a road surface: a dirt
//! road crosses a creek on a culvert or a log bridge, a paved road on a Bailey or a concrete beam
//! bridge, a railway on a girder or truss. Table: `assets/config/infrastructure/bridge_types.ron`
//! (built-in defaults when missing).
//!
//! Real low-cost families, with first-pass limits:
//! - **Culvert crossing** — earth fill over pipes / box culvert; short and shallow; built to be
//!   overtopped (closes in a flood, little damage).
//! - **Timber beam** — log/sawn stringers on two abutments; short single span.
//! - **Timber trestle** — timber deck on many timber bents; long but weak, piers catch debris.
//! - **Pontoon** — floating deck on pontoons; any depth, rides the water level, fails in fast
//!   current; roads only.
//! - **Bailey** — prefabricated steel panel bridge; single span up to ~60 m, fast to build.
//! - **Steel girder** — rolled/plate girders on concrete piers.
//! - **Prestressed concrete beam** — precast beams on piers; the sturdy standard.
//! - **Steel truss** — long spans, few piers in the channel; the classic rail crossing.
//! - **Stone arch** — masonry arches; slow to build, very durable.

use std::sync::OnceLock;

use serde::Deserialize;

use super::RoadTier;

/// Bridge structure family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum BridgeType {
    Culvert,
    TimberBeam,
    TimberTrestle,
    Pontoon,
    Bailey,
    SteelGirder,
    PrestressedConcrete,
    SteelTruss,
    StoneArch,
}

impl BridgeType {
    pub const ALL: [BridgeType; 9] = [
        BridgeType::Culvert,
        BridgeType::TimberBeam,
        BridgeType::TimberTrestle,
        BridgeType::Pontoon,
        BridgeType::Bailey,
        BridgeType::SteelGirder,
        BridgeType::PrestressedConcrete,
        BridgeType::SteelTruss,
        BridgeType::StoneArch,
    ];

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            BridgeType::Culvert => "culvert crossing",
            BridgeType::TimberBeam => "timber beam bridge",
            BridgeType::TimberTrestle => "timber trestle",
            BridgeType::Pontoon => "pontoon bridge",
            BridgeType::Bailey => "Bailey bridge",
            BridgeType::SteelGirder => "steel girder bridge",
            BridgeType::PrestressedConcrete => "concrete beam bridge",
            BridgeType::SteelTruss => "steel truss bridge",
            BridgeType::StoneArch => "stone arch bridge",
        }
    }

    #[must_use]
    pub fn spec(self) -> &'static BridgeTypeSpec {
        bridge_type_table().spec(self)
    }
}

/// Limits and flood behaviour of one bridge type.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BridgeTypeSpec {
    pub kind: BridgeType,
    /// Longest crossing (m).
    pub max_length_m: f32,
    /// Deepest water it can be founded in (m); floating types ignore it.
    pub max_depth_m: f32,
    /// Pier spacing (m); a crossing no longer than this is one span with no piers in the water.
    pub span_m: f32,
    /// Load (water depth m × speed² m²/s²) the structure takes without damage.
    pub strength: f32,
    /// Deck soffit above the water at build time (m).
    pub clearance_m: f32,
    /// Damage multiplier when water goes over the deck (culverts are built to be overtopped).
    pub overtop_damage_mul: f32,
    /// Rides the water level (never overtopped); fails above `max_current_mps`.
    pub floating: bool,
    pub max_current_mps: f32,
    /// Blast damage (warhead peak × falloff) that takes one section from whole to down.
    pub blast_toughness: f32,
    /// Materials per metre of bridge (cargo, units) — for the construction bill.
    pub materials_per_m: Vec<(String, f32)>,
}

impl BridgeTypeSpec {
    /// Share of the river load the structure catches: piers in the channel catch it all, a
    /// single span (abutments on the banks) much less.
    #[must_use]
    pub fn pier_exposure(&self, length_m: f32) -> f32 {
        if length_m <= self.span_m { 0.35 } else { 1.0 }
    }
}

/// The table.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BridgeTypeTable {
    pub schema_version: u32,
    pub types: Vec<BridgeTypeSpec>,
}

impl BridgeTypeTable {
    #[must_use]
    pub fn spec(&self, t: BridgeType) -> &BridgeTypeSpec {
        self.types
            .iter()
            .find(|s| s.kind == t)
            .unwrap_or_else(|| default_table().types.iter().find(|s| s.kind == t).expect("default has all types"))
    }
}

const TABLE_PATH: &str = "assets/config/infrastructure/bridge_types.ron";

#[allow(clippy::too_many_arguments)]
fn row(kind: BridgeType, len: f32, depth: f32, span: f32, strength: f32, clear: f32, over: f32, floating: bool, current: f32, tough: f32, mats: &[(&str, f32)]) -> BridgeTypeSpec {
    BridgeTypeSpec {
        kind,
        max_length_m: len,
        max_depth_m: depth,
        span_m: span,
        strength,
        clearance_m: clear,
        overtop_damage_mul: over,
        floating,
        max_current_mps: current,
        blast_toughness: tough,
        materials_per_m: mats.iter().map(|(c, n)| ((*c).to_owned(), *n)).collect(),
    }
}

fn default_table() -> &'static BridgeTypeTable {
    static DEFAULT: OnceLock<BridgeTypeTable> = OnceLock::new();
    DEFAULT.get_or_init(|| BridgeTypeTable {
        schema_version: 1,
        types: vec![
            row(BridgeType::Culvert, 16.0, 3.0, 16.0, 40.0, 0.5, 0.2, false, 0.0, 150.0, &[("Gravel", 6.0), ("Concrete", 1.0)]),
            row(BridgeType::TimberBeam, 24.0, 4.0, 12.0, 35.0, 2.0, 1.0, false, 0.0, 40.0, &[("Timber", 2.0)]),
            row(BridgeType::TimberTrestle, 150.0, 8.0, 6.0, 30.0, 3.0, 1.0, false, 0.0, 35.0, &[("Timber", 3.0)]),
            row(BridgeType::Pontoon, 300.0, 100.0, 300.0, 0.0, 0.0, 0.0, true, 2.0, 30.0, &[("Steel", 0.6), ("Timber", 0.5)]),
            row(BridgeType::Bailey, 200.0, 8.0, 60.0, 120.0, 3.0, 1.0, false, 0.0, 70.0, &[("Steel", 1.2)]),
            row(BridgeType::SteelGirder, 600.0, 25.0, 40.0, 250.0, 5.0, 1.0, false, 0.0, 120.0, &[("Steel", 2.5), ("Concrete", 3.0)]),
            row(BridgeType::PrestressedConcrete, 1000.0, 30.0, 35.0, 400.0, 6.0, 1.0, false, 0.0, 160.0, &[("Concrete", 8.0), ("Steel", 1.0)]),
            row(BridgeType::SteelTruss, 400.0, 30.0, 100.0, 300.0, 8.0, 1.0, false, 0.0, 110.0, &[("Steel", 4.0), ("Concrete", 2.0)]),
            row(BridgeType::StoneArch, 80.0, 6.0, 20.0, 350.0, 3.0, 0.6, false, 0.0, 200.0, &[("Limestone", 12.0), ("Cement", 1.0)]),
        ],
    })
}

/// The loaded table (RON when present and valid, else the built-in defaults).
#[must_use]
pub fn bridge_type_table() -> &'static BridgeTypeTable {
    static TABLE: OnceLock<BridgeTypeTable> = OnceLock::new();
    TABLE.get_or_init(|| {
        std::fs::read_to_string(TABLE_PATH)
            .ok()
            .and_then(|text| match ron::from_str::<BridgeTypeTable>(&text) {
                Ok(t) => Some(t),
                Err(e) => {
                    bevy::log::warn!(target: "infrastructure::bridges", "bridge type table {TABLE_PATH}: {e}; using defaults");
                    None
                }
            })
            .unwrap_or_else(|| default_table().clone())
    })
}

/// What carries the crossing: a road of some tier, or rail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BridgeCarries {
    Road(RoadTier),
    Rail,
}

/// Types a line would build, cheapest first.
#[must_use]
pub fn bridge_type_preference(carries: BridgeCarries) -> &'static [BridgeType] {
    use BridgeType::*;
    match carries {
        BridgeCarries::Road(RoadTier::Track | RoadTier::Dirt) => &[Culvert, TimberBeam, TimberTrestle, Pontoon, Bailey],
        BridgeCarries::Road(RoadTier::Gravel) => &[Culvert, TimberBeam, Bailey, TimberTrestle, Pontoon, SteelGirder],
        // Bailey is a temporary / military bridge: gravel and dirt roads only.
        BridgeCarries::Road(RoadTier::Paved) => &[Culvert, PrestressedConcrete, SteelGirder, SteelTruss],
        BridgeCarries::Road(RoadTier::Concrete) => &[Culvert, PrestressedConcrete, SteelGirder, SteelTruss],
        BridgeCarries::Road(RoadTier::Motorway) => &[Culvert, PrestressedConcrete, SteelGirder, SteelTruss],
        BridgeCarries::Rail => &[Culvert, StoneArch, SteelGirder, SteelTruss, PrestressedConcrete],
    }
}

/// Cheapest type that spans `length_m` of water up to `depth_m` deep for this line, if any.
#[must_use]
pub fn choose_bridge_type(carries: BridgeCarries, length_m: f32, depth_m: f32) -> Option<BridgeType> {
    bridge_type_preference(carries).iter().copied().find(|t| {
        let s = t.spec();
        length_m <= s.max_length_m && (s.floating || depth_m <= s.max_depth_m)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_table_parses_and_has_every_type() {
        let text = std::fs::read_to_string(TABLE_PATH).expect("bridge type table ships");
        let table: BridgeTypeTable = ron::from_str(&text).expect("valid RON");
        for t in BridgeType::ALL {
            assert!(table.types.iter().any(|s| s.kind == t), "{t:?} missing");
        }
    }

    #[test]
    fn crossings_get_the_cheapest_type_that_fits() {
        let dirt = BridgeCarries::Road(RoadTier::Dirt);
        assert_eq!(choose_bridge_type(dirt, 10.0, 1.0), Some(BridgeType::Culvert), "creek: culvert");
        assert_eq!(choose_bridge_type(dirt, 20.0, 3.5), Some(BridgeType::TimberBeam));
        assert_eq!(choose_bridge_type(dirt, 120.0, 3.0), Some(BridgeType::TimberTrestle));
        assert_eq!(choose_bridge_type(dirt, 250.0, 20.0), Some(BridgeType::Pontoon), "wide deep river: float it");
        assert_eq!(choose_bridge_type(BridgeCarries::Road(RoadTier::Paved), 120.0, 3.0), Some(BridgeType::PrestressedConcrete));
        assert_eq!(choose_bridge_type(BridgeCarries::Rail, 300.0, 10.0), Some(BridgeType::SteelGirder));
        assert_eq!(choose_bridge_type(BridgeCarries::Rail, 2000.0, 10.0), None, "nothing spans 2 km");
    }
}
