use crate::idgen::*;

pub trait AgentOwnable {
    fn set_owner(&mut self, owner_id: EntityId);
    fn get_owner(&self) -> EntityId;
}
