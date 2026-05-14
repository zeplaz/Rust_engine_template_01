//! Versioned bind-group cache keyed by [`BindGroupId`]; entries invalidate when bound buffer versions change.

use std::collections::HashMap;

use bevy::prelude::Resource;
use bevy::render::render_resource::{BindGroup, BufferBinding};

use super::gpu_buffer_registry::{BufferId, GPUBufferRegistry};

/// Stable numeric identity for a cached bind group (no string keys on the hot path).
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct BindGroupId(pub u64);

/// Weather/fire field compute pass: packed fire instance storage (`weather_fire_field.wgsl` group 1).
pub const WEATHER_FIRE_FIELD_FIRE_BIND_GROUP: BindGroupId = BindGroupId(1);
/// World fire particle instancing compute pass (`fire_particle.wgsl` group 1).
pub const WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP: BindGroupId = BindGroupId(2);
/// Expanded instanced-quad vertices (`fire_particle.wgsl` group 2).
pub const WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP: BindGroupId = BindGroupId(3);

/// Buffer dependency recorded when a bind group was built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BindGroupBufferBinding {
    pub buffer_id: BufferId,
    pub buffer_version: u64,
}

#[derive(Debug, Clone)]
pub struct GPUBindGroupEntry {
    pub bind_group: BindGroup,
    pub bindings: Vec<BindGroupBufferBinding>,
    pub version: u64,
}

#[derive(Resource, Default)]
pub struct GPUBindGroupRegistry {
    entries: HashMap<BindGroupId, GPUBindGroupEntry>,
    next_version: u64,
}

impl GPUBindGroupRegistry {
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn get(&self, id: BindGroupId) -> Option<&GPUBindGroupEntry> {
        self.entries.get(&id)
    }

    /// Returns `true` when cached bindings still match the live buffer versions.
    #[must_use]
    pub fn is_valid(&self, id: BindGroupId, registry: &GPUBufferRegistry) -> bool {
        let Some(entry) = self.entries.get(&id) else {
            return false;
        };
        entry.bindings.iter().all(|binding| {
            registry
                .get(binding.buffer_id)
                .is_some_and(|buffer| buffer.version == binding.buffer_version)
        })
    }

    pub fn invalidate(&mut self, id: BindGroupId) {
        self.entries.remove(&id);
    }

    pub fn insert(
        &mut self,
        id: BindGroupId,
        bind_group: BindGroup,
        bindings: Vec<BindGroupBufferBinding>,
    ) {
        self.next_version = self.next_version.wrapping_add(1);
        self.entries.insert(
            id,
            GPUBindGroupEntry {
                bind_group,
                bindings,
                version: self.next_version,
            },
        );
    }
}

#[must_use]
pub fn buffer_binding_for(registry: &GPUBufferRegistry, id: BufferId) -> Option<BufferBinding<'_>> {
    let entry = registry.get(id)?;
    Some(BufferBinding {
        buffer: &entry.buffer,
        offset: 0,
        size: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_group_id_ordering_is_stable() {
        assert!(BindGroupId(1) < BindGroupId(2));
        assert_eq!(WEATHER_FIRE_FIELD_FIRE_BIND_GROUP, BindGroupId(1));
        assert_eq!(WORLD_FIRE_PARTICLE_DRAW_BIND_GROUP, BindGroupId(2));
        assert_eq!(WORLD_FIRE_PARTICLE_EXPANDED_BIND_GROUP, BindGroupId(3));
    }
}
