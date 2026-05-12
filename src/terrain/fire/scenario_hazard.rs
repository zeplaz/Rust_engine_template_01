//! Editable **scenario / mission** hazard facets for fire ecology (`base_fire_sim.md` §8).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScenarioHazardLayer {
    WildfireRisk,
    FuelStorage,
    AmmoStorage,
    ChemicalHazard,
    SmokeZone,
    EvacuationRisk,
}
