// Core production utilities
mod building_core;
mod logistics_site;
mod resources;
mod manufacturing;
mod manufacturing_plugin;
mod production_care;
mod production_utils;

// Public exports
pub use building_core::*;
pub use logistics_site::*;
pub use resources::*;
pub use manufacturing::{
    is_deployable_staging_buffer_tag, ManufacturingBlueprint, ManufacturingDomain,
    ManufacturingNode, ManufacturingOutputBuffers, BUFFER_TAG_DRAGON_TEETH_UNIT,
    BUFFER_TAG_MINE_UNIT, MFG_DRAGON_TEETH_V1, MFG_MINE_UNIT_V1,
};
pub use manufacturing_plugin::{
    manufacturing_blueprint_id_for_catalog, manufacturing_deployable_bundle,
    manufacturing_node_for_catalog, tick_manufacturing_nodes, ManufacturingBlueprintRegistry,
    ManufacturingCorePlugin,
};
pub use production_care::*;
pub use production_utils::{
    categorize_resources, resource_category_of, resource_category_tag, ResourceCategory,
};
