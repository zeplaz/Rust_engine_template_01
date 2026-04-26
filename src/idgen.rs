// ID Generation
use std::sync::atomic::{AtomicU32, Ordering};
use serde::{Deserialize, Serialize};

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(u32);

impl EntityId {
    pub fn new() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
    
    pub fn from_u32(id: u32) -> Self {
        Self(id)
    }
    
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

// Reset the ID counter (useful for tests)
pub fn reset_id_counter() {
    NEXT_ID.store(1, Ordering::Relaxed);
}

// Get the current max ID (useful for debugging)
pub fn get_max_id() -> u32 {
    NEXT_ID.load(Ordering::Relaxed)
}