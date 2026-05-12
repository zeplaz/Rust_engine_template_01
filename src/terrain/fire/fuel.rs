//! Physical **fuel taxonomy** — combustible classes and baseline combustion parameters (design: `base_fire_sim.md`).
//! Normalized energies and rates are sim units `[0, 1]` unless noted (`ignition_temp_c` in °C for future coupling).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FuelMaterialKind {
    Grass,
    Brush,
    Timber,
    Peat,

    Coal,
    Oil,
    Diesel,
    Gasoline,

    WoodStructure,
    ConcreteStructure,
    SteelStructure,

    Ammunition,
    ChemicalOxidizer,
    BatteryLithium,
    Plastic,

    Rubber,
    Fabric,
}

#[derive(Clone, Copy, Debug)]
pub struct FuelMaterialDef {
    pub ignition_temp_c: f32,
    pub burn_energy: f32,
    pub burn_duration: f32,
    pub smoke_density: f32,
    pub toxic_output: f32,
   
    /// Normalized overpressure proxy for logistics / mission hazard.
    pub explosive_force: f32,
    pub contamination: f32,
    pub thermal_runaway: bool,
    pub structural_damage_rate: f32,
}

pub fn fuel_material_def(kind: FuelMaterialKind) -> FuelMaterialDef {
    match kind {
        FuelMaterialKind::Grass => FuelMaterialDef {
            ignition_temp_c: 230.0,
            burn_energy: 0.35,
            burn_duration: 12.0,
            smoke_density: 0.15,
            toxic_output: 0.0,
            explosive_force: 0.0,
            contamination: 0.0,
            thermal_runaway: false,
            structural_damage_rate: 0.0,
        },
        FuelMaterialKind::Brush => FuelMaterialDef {
            ignition_temp_c: 260.0,
            burn_energy: 0.45,
            burn_duration: 35.0,
            smoke_density: 0.35,
            toxic_output: 0.05,
            explosive_force: 0.0,
            contamination: 0.0,
            thermal_runaway: false,
            structural_damage_rate: 0.15,
        },
        FuelMaterialKind::Timber => FuelMaterialDef {
            ignition_temp_c: 300.0,
            burn_energy: 0.7,
            burn_duration: 120.0,
            smoke_density: 0.5,
            toxic_output: 0.1,
            explosive_force: 0.0,
            contamination: 0.0,
            thermal_runaway: false,
            structural_damage_rate: 0.4,
        },
        FuelMaterialKind::Peat => FuelMaterialDef {
            ignition_temp_c: 280.0,
            burn_energy: 0.55,
            burn_duration: 400.0,
            smoke_density: 0.85,
            toxic_output: 0.2,
            explosive_force: 0.0,
            contamination: 0.35,
            thermal_runaway: false,
            structural_damage_rate: 0.1,
        },

        FuelMaterialKind::Coal => FuelMaterialDef {
            ignition_temp_c: 350.0,
            burn_energy: 0.8,
            burn_duration: 300.0,
            smoke_density: 0.95,
            toxic_output: 0.45,
            explosive_force: 0.0,
            contamination: 0.55,
            thermal_runaway: false,
            structural_damage_rate: 0.25,
        },
        FuelMaterialKind::Oil => FuelMaterialDef {
            ignition_temp_c: 200.0,
            burn_energy: 0.95,
            burn_duration: 80.0,
            smoke_density: 0.75,
            toxic_output: 0.5,
            explosive_force: 0.35,
            contamination: 0.8,
            thermal_runaway: false,
            structural_damage_rate: 0.9,
        },
        FuelMaterialKind::Diesel => FuelMaterialDef {
            ignition_temp_c: 220.0,
            burn_energy: 0.9,
            burn_duration: 90.0,
            smoke_density: 0.92,
            toxic_output: 0.55,
            explosive_force: 0.55,
            contamination: 0.75,
            thermal_runaway: false,
            structural_damage_rate: 0.95,
        },
        FuelMaterialKind::Gasoline => FuelMaterialDef {
            ignition_temp_c: 210.0,
            burn_energy: 1.0,
            burn_duration: 40.0,
            smoke_density: 0.8,
            toxic_output: 0.3,
            explosive_force: 0.85,
            contamination: 0.5,
            thermal_runaway: false,
            structural_damage_rate: 1.0,
        },

        FuelMaterialKind::WoodStructure => FuelMaterialDef {
            ignition_temp_c: 290.0,
            burn_energy: 0.65,
            burn_duration: 180.0,
            smoke_density: 0.55,
            toxic_output: 0.12,
            explosive_force: 0.05,
            contamination: 0.05,
            thermal_runaway: false,
            structural_damage_rate: 0.55,
        },
        FuelMaterialKind::ConcreteStructure => FuelMaterialDef {
            ignition_temp_c: 900.0,
            burn_energy: 0.12,
            burn_duration: 600.0,
            smoke_density: 0.2,
            toxic_output: 0.0,
            explosive_force: 0.0,
            contamination: 0.02,
            thermal_runaway: false,
            structural_damage_rate: 0.15,
        },
        FuelMaterialKind::SteelStructure => FuelMaterialDef {
            ignition_temp_c: 850.0,
            burn_energy: 0.25,
            burn_duration: 400.0,
            smoke_density: 0.35,
            toxic_output: 0.05,
            explosive_force: 0.0,
            contamination: 0.03,
            thermal_runaway: false,
            structural_damage_rate: 0.35,
        },

        FuelMaterialKind::Ammunition => FuelMaterialDef {
            ignition_temp_c: 170.0,
            burn_energy: 0.9,
            burn_duration: 60.0,
            smoke_density: 0.4,
            toxic_output: 0.2,
            explosive_force: 1.0,
            contamination: 0.1,
            thermal_runaway: true,
            structural_damage_rate: 1.0,
        },
        FuelMaterialKind::ChemicalOxidizer => FuelMaterialDef {
            ignition_temp_c: 140.0,
            burn_energy: 0.85,
            burn_duration: 90.0,
            smoke_density: 0.45,
            toxic_output: 1.0,
            explosive_force: 0.9,
            contamination: 0.95,
            thermal_runaway: true,
            structural_damage_rate: 0.85,
        },
        FuelMaterialKind::BatteryLithium => FuelMaterialDef {
            ignition_temp_c: 120.0,
            burn_energy: 0.95,
            burn_duration: 400.0,
            smoke_density: 1.0,
            toxic_output: 0.95,
            explosive_force: 0.25,
            contamination: 0.7,
            thermal_runaway: true,
            structural_damage_rate: 0.8,
        },
        FuelMaterialKind::Plastic => FuelMaterialDef {
            ignition_temp_c: 320.0,
            burn_energy: 0.6,
            burn_duration: 70.0,
            smoke_density: 0.95,
            toxic_output: 0.65,
            explosive_force: 0.15,
            contamination: 0.45,
            thermal_runaway: false,
            structural_damage_rate: 0.35,
        },
        FuelMaterialKind::Rubber => FuelMaterialDef {
            ignition_temp_c: 260.0,
            burn_energy: 0.55,
            burn_duration: 55.0,
            smoke_density: 0.88,
            toxic_output: 0.5,
            explosive_force: 0.1,
            contamination: 0.35,
            thermal_runaway: false,
            structural_damage_rate: 0.3,
        },
        FuelMaterialKind::Fabric => FuelMaterialDef {
            ignition_temp_c: 250.0,
            burn_energy: 0.4,
            burn_duration: 25.0,
            smoke_density: 0.55,
            toxic_output: 0.08,
            explosive_force: 0.0,
            contamination: 0.02,
            thermal_runaway: false,
            structural_damage_rate: 0.1,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gasoline_more_explosive_than_grass() {
        let g = fuel_material_def(FuelMaterialKind::Gasoline);
        let grass = fuel_material_def(FuelMaterialKind::Grass);
        assert!(g.explosive_force > grass.explosive_force);
    }
}
