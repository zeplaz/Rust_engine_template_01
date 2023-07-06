use bevy::utils::HashMap;

use crate::entities::prelude::*;


pub type ResourceRequirement = (f32, f32);
pub type ResourceRequirementsMap = HashMap<ResourceType, ResourceRequirement>;
