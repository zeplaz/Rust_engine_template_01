
use crate::idgen::*;

pub trait Identifiable {
    fn id(&self) -> EntityId;
}
