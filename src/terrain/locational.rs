use std::collections::HashMap;

use crate::idgen::EntityId;

/// Cell-keyed spatial index (entity IDs per grid cell).
#[derive(Debug, Clone)]
pub struct CellMap2d<T: Copy> {
    pub cell_size: T,
    pub cells: HashMap<(i32, i32), Vec<EntityId>>,
}

impl<T: Copy> CellMap2d<T> {
    pub fn new(cell_size: T) -> Self {
        Self {
            cell_size,
            cells: HashMap::default(),
        }
    }
}
