

pub mod prelude;
//baseic
pub mod types_aliases; 
pub mod entity;

//the ez entiy paticlar attrabutes
pub mod e_infos; 
pub mod e_states;
pub mod e_flag_types;
pub mod e_componets;

//submods for all the major entities in game
pub mod damages;
pub mod production;
pub mod strukturave;
pub mod vehicles;
//pub mod prefab_cats;

//goina make those pub so easy to use
pub use damages::*;


//make basic pub
pub use entity::*;
pub use prelude::*;

