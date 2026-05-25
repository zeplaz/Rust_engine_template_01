//! Building **catalog** — designer-facing defs from legacy asset tools + FlatBuffers.
//!
//! Source of truth for labels and structure (not yet full runtime sim):
//! - [`schemas/flatbuffers/skrukturave_01.fbs`] (`ResidenceType`, `ApartmentUnitUnion`, `ApartmentUnitType`)
//! - [`utils/asset_tools/src/templates/buildings.py`]
//! - [`assets/configs/buildings/_building_types_index.json`]
//! - [`assets/configs/buildings/*.json`] (footprint, construction_cost, power, production)
//!
//! **Excluded from gameplay preview** (not part of construction UX): land value, housing value,
//! abstract “market” scores — those are not operational build parameters.

/// Footprint occupancy on the build grid (legacy `BuildingMatrixGrid`).
#[derive(Clone, Debug, Default)]
pub struct FootprintMatrix {
    pub width: u32,
    pub depth: u32,
    /// Row-major 0/1 occupancy.
    pub cells: Vec<u8>,
}

impl FootprintMatrix {
    #[must_use]
    pub fn from_size(width: u32, depth: u32, filled: bool) -> Self {
        let n = (width * depth) as usize;
        Self {
            width,
            depth,
            cells: vec![u8::from(filled); n],
        }
    }
}

/// Unit layout inside an apartment building (legacy `APARTMENT_UNIT_TYPES`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApartmentUnitKind {
    Studio,
    Single,
    Double,
    ThreeBedrooms,
    Family,
    Luxury,
}

/// Multi-unit residential **form** (legacy `APARTMENT_TYPES` / FBS `ApartmentUnitUnion`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApartmentForm {
    HighRise,
    Duplex,
    Quadplex,
    ThreeStoryBlock,
    FiveStoryBlock,
}

impl ApartmentForm {
    /// Max countable units enforced in legacy asset-tools UI.
    #[must_use]
    pub const fn max_units(self) -> Option<u32> {
        match self {
            Self::Duplex => Some(2),
            Self::Quadplex => Some(4),
            Self::HighRise | Self::ThreeStoryBlock | Self::FiveStoryBlock => None,
        }
    }
}

/// Detached / estate residential (legacy `RESEDENCY_TYPES` / FBS `ResidenceType`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetachedResidenceForm {
    SmallHouse,
    MediumHouse,
    LargeHouse,
    Estate,
}

/// Full residential pick: detached home or multi-unit block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidentialBuildingForm {
    Detached(DetachedResidenceForm),
    Apartments(ApartmentForm),
}

/// Top-level building families from legacy `BUILDING_TYPES` / tool index.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BuildingFamily {
    #[default]
    Residential,
    Retail,
    Civic,
    Logistics,
    Industry,
    Extraction,
    Fuel,
    Power,
    Research,
    Agriculture,
    Rail,
}

/// Operational preview fields shown before commit (intent panel) — **no** land/housing value.
#[derive(Clone, Debug, Default)]
pub struct BuildingIntentPreview {
    pub label: String,
    pub family: BuildingFamily,
    pub footprint: FootprintMatrix,
    pub construction_cost: u32,
    pub construction_time_ticks: u32,
    pub power_consumption: f32,
    pub workers_required: u32,
    /// Residential only: unit mix summary for UI.
    pub unit_kinds: Vec<ApartmentUnitKind>,
    pub apartment_form: Option<ApartmentForm>,
    /// Round 3 catalog id (`assets/configs/buildings` stem or `builtin:*`).
    pub catalog_id: Option<String>,
}

#[must_use]
pub fn default_preview_for_apartment(form: ApartmentForm) -> BuildingIntentPreview {
    let (label, units) = match form {
        ApartmentForm::Duplex => ("Duplex", vec![ApartmentUnitKind::Single, ApartmentUnitKind::Single]),
        ApartmentForm::Quadplex => (
            "Quadplex",
            vec![
                ApartmentUnitKind::Single,
                ApartmentUnitKind::Single,
                ApartmentUnitKind::Single,
                ApartmentUnitKind::Single,
            ],
        ),
        ApartmentForm::HighRise => ("High-rise", vec![ApartmentUnitKind::Studio, ApartmentUnitKind::Family]),
        ApartmentForm::ThreeStoryBlock => ("3-story block", vec![ApartmentUnitKind::Double]),
        ApartmentForm::FiveStoryBlock => ("5-story block", vec![ApartmentUnitKind::Family]),
    };
    BuildingIntentPreview {
        label: label.into(),
        family: BuildingFamily::Residential,
        footprint: FootprintMatrix::from_size(2, 2, true),
        construction_cost: 240,
        construction_time_ticks: 120,
        power_consumption: 12.0,
        workers_required: 0,
        unit_kinds: units,
        apartment_form: Some(form),
        catalog_id: None,
    }
}
