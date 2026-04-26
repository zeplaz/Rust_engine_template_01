use crate::entities::prelude::*;
use crate::entities::types_of::ResourceType;
use bevy::utils::HashMap;

pub type ResourceRequirement = (f32, f32);
pub type ResourceRequirementsMap = HashMap<ResourceType, ResourceRequirement>;
