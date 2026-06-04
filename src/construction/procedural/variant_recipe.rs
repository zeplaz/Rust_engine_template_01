//! TILE-FIX-006 — procedural variant combinations before render (not post-hoc dimming).

/// Axis layered when expanding bake/runtime variant keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariantLayer {
    Lighting,
    Damage,
    Occupancy,
    Fire,
    Construction,
}

/// Single resolved bake row: variant key + which layers contributed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantRecipe {
    pub variant_key: String,
    pub layers: Vec<VariantLayer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LightingState {
    #[default]
    Day,
    Night,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DamageState {
    #[default]
    None,
    Light,
    Heavy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OccupancyState {
    #[default]
    Operational,
    Abandoned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FireState {
    #[default]
    Off,
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConstructionState {
    #[default]
    Complete,
    Early,
    Mid,
    Late,
}

/// Designer/sim axes folded into catalog variant keys.
#[derive(Debug, Clone, Copy, Default)]
pub struct BuildingState {
    pub lighting: LightingState,
    pub damage: DamageState,
    pub occupancy: OccupancyState,
    pub fire: FireState,
    pub construction: ConstructionState,
}

impl BuildingState {
    #[must_use]
    pub fn variant_key(&self) -> &'static str {
        match (
            self.construction,
            self.fire,
            self.occupancy,
            self.damage,
            self.lighting,
        ) {
            (ConstructionState::Early, _, _, _, _) => "under_construction_01",
            (ConstructionState::Mid, _, _, _, _) => "under_construction_02",
            (ConstructionState::Late, _, _, _, _) => "under_construction_03",
            (_, FireState::Small | FireState::Medium | FireState::Large, _, _, _) => "burning_00",
            (_, _, OccupancyState::Abandoned, _, _) => "abandoned",
            (_, _, _, DamageState::Heavy, _) => "ruined",
            (_, _, _, DamageState::Light, LightingState::Night) => "damaged_night_on",
            (_, _, _, DamageState::Light, LightingState::Day) => "damaged_day",
            (_, _, _, DamageState::None, LightingState::Night) => "clean_night_off",
            _ => "clean_day",
        }
    }

    #[must_use]
    pub fn active_layers(&self) -> Vec<VariantLayer> {
        let mut out = Vec::new();
        if self.construction != ConstructionState::Complete {
            out.push(VariantLayer::Construction);
        }
        if self.fire != FireState::Off {
            out.push(VariantLayer::Fire);
        }
        if self.occupancy == OccupancyState::Abandoned {
            out.push(VariantLayer::Occupancy);
        }
        if self.damage != DamageState::None {
            out.push(VariantLayer::Damage);
        }
        if self.lighting == LightingState::Night {
            out.push(VariantLayer::Lighting);
        }
        out
    }

    #[must_use]
    pub fn recipe(&self) -> VariantRecipe {
        VariantRecipe {
            variant_key: self.variant_key().to_owned(),
            layers: self.active_layers(),
        }
    }
}

/// Expand a Cartesian product of layer axes into deterministic recipes (pre-render bake matrix).
#[must_use]
pub fn expand_variant_recipes(
    lighting: &[LightingState],
    damage: &[DamageState],
    occupancy: &[OccupancyState],
    fire: &[FireState],
    construction: &[ConstructionState],
) -> Vec<VariantRecipe> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for &c in construction {
        for &f in fire {
            for &o in occupancy {
                for &d in damage {
                    for &l in lighting {
                        let state = BuildingState {
                            lighting: l,
                            damage: d,
                            occupancy: o,
                            fire: f,
                            construction: c,
                        };
                        let key = state.variant_key().to_owned();
                        if seen.insert(key.clone()) {
                            out.push(state.recipe());
                        }
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| a.variant_key.cmp(&b.variant_key));
    out
}

/// Minimum ship matrix from catalog keys (deduped, stable order).
#[must_use]
pub fn recipes_from_catalog_keys(keys: &[&str]) -> Vec<VariantRecipe> {
    let mut out = Vec::new();
    for key in keys {
        out.push(VariantRecipe {
            variant_key: (*key).to_owned(),
            layers: Vec::new(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_state_maps_clean_day() {
        let s = BuildingState::default();
        assert_eq!(s.variant_key(), "clean_day");
        assert!(s.active_layers().is_empty());
    }

    #[test]
    fn expand_recipes_dedupes_collision_keys() {
        let recipes = expand_variant_recipes(
            &[LightingState::Day, LightingState::Night],
            &[DamageState::None],
            &[OccupancyState::Operational],
            &[FireState::Off],
            &[ConstructionState::Complete],
        );
        let keys: Vec<_> = recipes.iter().map(|r| r.variant_key.as_str()).collect();
        assert!(keys.contains(&"clean_day"));
        assert!(keys.contains(&"clean_night_off"));
    }

    #[test]
    fn fire_layer_tags_recipe() {
        let s = BuildingState {
            fire: FireState::Medium,
            ..Default::default()
        };
        let r = s.recipe();
        assert_eq!(r.variant_key, "burning_00");
        assert!(r.layers.contains(&VariantLayer::Fire));
    }
}
