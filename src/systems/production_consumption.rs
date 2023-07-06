use bevy::prelude::*;

//use crate::systems::power_systems::*;
use crate::entities::e_componets::*;

use crate::entities::damages::BuildingDamageInfo;
use crate::entities::e_states::ConstructionStates;
use crate::entities::e_flag_types::ResourceType;
pub struct ProductionPlugin{}

impl Plugin for ProductionPlugin {

    fn build(&self, app: &mut App)
    {
      // app.add_plugin();
    }


}


fn building_activity_system(
    mut query: Query<(
        &Building, 
        &mut ProductionComponent, 
        &mut ConsumptionComponent, 
        &mut ElectricalLoad,
        &BuildingDamageInfo,
    )>,
) {
    for (building, mut production, mut consumption, load, damage_info) in query.iter_mut() {
        // Skip buildings that are not operational
        if building.construction_state != ConstructionStates::Operational || 
                                          damage_info.structural_integrity < 0.5 {
            continue;
        }

        let building_productivity_factor = calculate_buildingdamage_productivity_rate(damage_info);
        let electricity_consumption_rate = (*consumption.consumption_rates.get(&ResourceType::Electricity).unwrap_or(&0.0))/load.current_load;

        electricity_consumption_rate = calculate_electrical_avilablity_quaility(electricity_consumption_rate);
        let productivity_factor = building_productivity_factor*electricity_consumption_rate*production.calculate_total_rate(); 
 

        if productivity_factor > production.max_rate{
            production.current_rate  =  production.max_rate;
        } 
        else if productivity_factor < 0 {
            production.current_rate = 0.0;
        }
        else {
            production.current_rate = productivity_factor; 
        }
       

        for (resource_type, production_rate) in &mut production.production_rate {
            // Apply damage and electrical supply effects to production rate
            *production_rate *= production.current_rate;           

            if let Some(consumption_rate) = consumption.consumption.get_mut(resource_type) {
                // Ensure there are enough resources to consume before producing
                let mut  pro_sotrage = *production.storage.get(resource_type).unwrap_or(&0.0);
                let mut comp_sotrage = *consumption.storage.get_mut(resource_type).unwrap();
                
                if *consumption_rate <= comp_sotrage{
                    // Deduct consumed resources from storage
                    comp_sotrage -= *consumption_rate*production_rate;
                    // Add produced resources to storage
                    *production.storage.get_mut(resource_type).unwrap() += *production_rate;
                }
                else if *consumption_rate > 0{
                    comp_sotrage =0;
                    *production.storage.get_mut(resource_type).unwrap() += *production_rate*(comp_sotrage / *consumption_rate);
                }
                 else {
                    // No goods
                    *comp_sotrage = 0.0;
                    *production_rate =0.0;
                }
            }
        }
    }
}

pub fn normalize(value:f32, min: f32, max: f32) -> f32 {
    let normalized= (value - min) / (max - min);
	return normalized;
}

#[allow(unused_parens)]
fn calculate_electrical_avilablity_quaility(availability: f32) -> f32 {
   
    let availability= normalize(availability, 0, 1);
    if availability == 0.0 {
        // Handle the special case where availability is zero to avoid division by zero error
        return 0.0;
    } else {
        return (availability * availability * availability);
    }
}


fn calculate_buildingdamage_productivity_rate(damage_info: &BuildingDamageInfo) -> f32 {
 

    let machinery_damage_factor = 1.0 - 0.5 * damage_info.machinery_damage;  
    let structural_integrity_factor = 1.0 - damage_info.structural_integrity;

    let electrical_connection_quality_factor = 1.0 - damage_info.electrical_connection_quality/damage_info.electrical_connection_quality;

    let productivity_rate = machinery_damage_factor * structural_integrity_factor * electrical_connection_quality_factor;

    // ensure that the productivity rate is within a reasonable range
    if productivity_rate < 0.0 {
        0.0
    } else if productivity_rate > 1.0 {
        1.0
    } else {
        productivity_rate
    }
}



