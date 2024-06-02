pub mod prelude;
//baseic
pub mod entity;
pub mod types_aliases;

//the ez entiy paticlar attrabutes
pub mod e_componets;
pub mod e_infos;
pub mod e_states;
pub mod types_of;

//submods for all the major entities in game
pub mod production;
pub mod strukturave;
pub mod vehicles;
//pub mod prefab_cats;

//goina make those pub so easy to use

pub use types_of::*;
//make basic pub
pub use entity::*;
pub use prelude::*;
