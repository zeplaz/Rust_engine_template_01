trait Building {
    fn is_producer(&self) -> bool;
    fn is_consumer(&self) -> bool;
    fn has_storage(&self) -> bool;
    fn get_building_type(&self) -> BuildingType;
    // other methods specific to buildings
}
