//! Optional **marker components** so failures / derates are independent but connected:
//! only entities with `SteamCycle` run steam-specific logic; only `ContainmentBuilding` runs nuclear pressure logic.

use bevy::prelude::*;

use crate::entities::production::power::components::PowerPlant;
use crate::entities::production::power::plant_profile::PlantArchetype;
use crate::entities::production::power::plant_registry::PlantDefinitionRegistry;

/// Thermal plant with a steam-water loop (coal, nuclear secondary, many gas/CHP, geothermal flash, etc.).
#[derive(Component, Debug, Clone, Copy)]
pub struct SteamCycle;

/// Nuclear island with containment / pressure boundary (meltdown, scram, containment breach — future systems).
#[derive(Component, Debug, Clone, Copy)]
pub struct ContainmentBuilding;

/// Intermittent renewable output (wind/solar curves, environmental shutdown already on `OperationalStatus`).
#[derive(Component, Debug, Clone, Copy)]
pub struct VariableRenewable;

#[derive(Clone, Copy)]
struct CapabilityFlagsResolved {
    steam: bool,
    containment: bool,
    variable: bool,
}

/// When a `PowerPlant` is added, attach markers from **`PlantDefinition`** when `definition_id` resolves;
/// otherwise from `PlantArchetype::for_type(plant_type)`.
pub fn attach_power_plant_capabilities(
    mut commands: Commands,
    defs: Res<PlantDefinitionRegistry>,
    q: Query<(Entity, &PowerPlant), Added<PowerPlant>>,
) {
    for (entity, plant) in &q {
        let flags = resolve_capabilities(plant, &defs);
        apply_capability_flags(&mut commands, entity, flags);
    }
}

fn resolve_capabilities(plant: &PowerPlant, defs: &PlantDefinitionRegistry) -> CapabilityFlagsResolved {
    if !plant.definition_id.is_empty() {
        if let Some(d) = defs.get(plant.definition_id.as_str()) {
            return CapabilityFlagsResolved {
                steam: d.capabilities.is_steam_cycle,
                containment: d.capabilities.is_nuclear_containment,
                variable: d.capabilities.is_variable_renewable,
            };
        }
    }
    let arch = PlantArchetype::for_type(plant.plant_type);
    CapabilityFlagsResolved {
        steam: arch.is_steam_cycle,
        containment: arch.is_nuclear_containment,
        variable: arch.is_variable_renewable,
    }
}

fn apply_capability_flags(commands: &mut Commands, entity: Entity, f: CapabilityFlagsResolved) {
    if f.steam {
        commands.entity(entity).insert(SteamCycle);
    } else {
        commands.entity(entity).remove::<SteamCycle>();
    }

    if f.containment {
        commands.entity(entity).insert(ContainmentBuilding);
    } else {
        commands.entity(entity).remove::<ContainmentBuilding>();
    }

    if f.variable {
        commands.entity(entity).insert(VariableRenewable);
    } else {
        commands.entity(entity).remove::<VariableRenewable>();
    }
}
