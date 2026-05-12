// Serialization and deserialization
mod deserializers;
mod legacy_drez;
mod resource_deserializer;

// Public exports
pub use deserializers::deserialize_road_vehicle_configs;
pub use resource_deserializer::*;