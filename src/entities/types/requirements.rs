use crate::entities::types::p_enumz::ResourceType;
use std::collections::HashMap;

pub type ResourceRequirement = (f32, f32);
pub type ResourceRequirementsMap = HashMap<ResourceType, ResourceRequirement>;
