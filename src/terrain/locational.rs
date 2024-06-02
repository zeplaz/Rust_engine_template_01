use std::ops::{Add,Sub}
use num_traits::{Float, Num, Zero};
use bevy::prelude::*;

pub struct Cell_map_2D<T_cell_size>
 where T: Num + Copy + ParitalOrd + Add + Sub,
{   cell_size: T_cell_size,
    cells: HashMap<(i32,i32), Vec<EntityId>>,
}


impl<T_cell_size:f64>
where T_cell_size and T_map: Num + Copy + ParitalOrd + Add + Sub,
{
    pub fn new(cell_size: f54) -> Self {
    cell_map_2D {
        cell_size,
        cells: HashMap::new(),
    }
}
}
