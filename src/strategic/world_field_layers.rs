//! Scalar **field layers** per cell (economy / control / sensing) — parallel to [`super::ChunkStrategicOverlay`]
//! but oriented toward unified planner reads ([`super::WorldReadSnapshot`]).
//!
//! Nodes and zones write **deltas** into these layers; systems blend, decay, and expose summaries.
//!
//! Update-rate tiers (`TAC-06`): each scalar maps to [`FieldLayerUpdateClass`]; [`WorldFieldLayerEpoch`]
//! holds one counter per class so writers bump only their tier and consumers invalidate against that tier.

use bevy::prelude::*;

/// Tactical analysis update rate (AIG 3e §6.2.5): static geography, slow political/morale fields,
/// dynamic combat / logistics / sensing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum FieldLayerUpdateClass {
    #[default]
    Static,
    Slow,
    Dynamic,
}

/// Logical scalar carried in [`ChunkFieldCell`] (one epoch class each).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FieldScalarLayer {
    Supply,
    Demand,
    Power,
    Control,
    Visibility,
    Threat,
    Morale,
}

impl FieldScalarLayer {
    /// Update-rate class for this scalar (writers bump [`WorldFieldLayerEpoch`] for this class only).
    #[inline]
    pub const fn update_class(self) -> FieldLayerUpdateClass {
        match self {
            Self::Power => FieldLayerUpdateClass::Static,
            Self::Control | Self::Morale => FieldLayerUpdateClass::Slow,
            Self::Supply | Self::Demand | Self::Visibility | Self::Threat => {
                FieldLayerUpdateClass::Dynamic
            }
        }
    }

    /// Read the scalar from a packed cell (for cache refresh closures).
    #[inline]
    pub fn read_from(self, cell: &ChunkFieldCell) -> f32 {
        match self {
            Self::Supply => cell.supply,
            Self::Demand => cell.demand,
            Self::Power => cell.power,
            Self::Control => cell.control,
            Self::Visibility => cell.visibility,
            Self::Threat => cell.threat,
            Self::Morale => cell.morale,
        }
    }
}

/// Packed cell — matches the “world as layered fields” mental model (single grid cell).
#[derive(Clone, Copy, Debug, Default)]
pub struct ChunkFieldCell {
    pub supply: f32,
    pub demand: f32,
    pub power: f32,
    pub control: f32,
    pub visibility: f32,
    pub threat: f32,
    pub morale: f32,
}

impl ChunkFieldCell {
    #[inline]
    pub fn add_scaled(&mut self, o: &Self, scale: f32) {
        self.supply += o.supply * scale;
        self.demand += o.demand * scale;
        self.power += o.power * scale;
        self.control += o.control * scale;
        self.visibility += o.visibility * scale;
        self.threat += o.threat * scale;
        self.morale += o.morale * scale;
    }
}

/// Per–update-class epochs bumped when field layers in that tier are recomputed.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldFieldLayerEpoch {
    pub static_epoch: u64,
    pub slow_epoch: u64,
    pub dynamic_epoch: u64,
}

impl WorldFieldLayerEpoch {
    #[inline]
    pub fn get(self, class: FieldLayerUpdateClass) -> u64 {
        match class {
            FieldLayerUpdateClass::Static => self.static_epoch,
            FieldLayerUpdateClass::Slow => self.slow_epoch,
            FieldLayerUpdateClass::Dynamic => self.dynamic_epoch,
        }
    }

    /// Writers call after recomputing all scalars in `class`.
    #[inline]
    pub fn bump(&mut self, class: FieldLayerUpdateClass) {
        match class {
            FieldLayerUpdateClass::Static => self.static_epoch = self.static_epoch.wrapping_add(1),
            FieldLayerUpdateClass::Slow => self.slow_epoch = self.slow_epoch.wrapping_add(1),
            FieldLayerUpdateClass::Dynamic => {
                self.dynamic_epoch = self.dynamic_epoch.wrapping_add(1)
            }
        }
    }

    #[inline]
    pub fn epoch_for_scalar(self, layer: FieldScalarLayer) -> u64 {
        self.get(layer.update_class())
    }

    #[inline]
    pub fn bump_for_scalar(&mut self, layer: FieldScalarLayer) {
        self.bump(layer.update_class());
    }
}

/// Consumer-side cache: refresh only when the scalar's update-class epoch moves.
#[derive(Clone, Copy, Debug)]
pub struct FieldLayerConsumerCache<T> {
    layer: FieldScalarLayer,
    seen_epoch: u64,
    value: T,
}

impl<T: Copy> FieldLayerConsumerCache<T> {
    #[inline]
    pub fn new(layer: FieldScalarLayer, initial: T) -> Self {
        Self {
            layer,
            // Force first `value_or_refresh` to call `fetch` (live epoch may also be 0).
            seen_epoch: u64::MAX,
            value: initial,
        }
    }

    #[inline]
    pub fn cached_value(self) -> T {
        self.value
    }

    #[inline]
    pub fn seen_epoch(self) -> u64 {
        self.seen_epoch
    }

    /// Returns cached value, refreshing from `fetch` when the tier epoch for `layer` changed.
    #[inline]
    pub fn value_or_refresh(
        &mut self,
        epochs: &WorldFieldLayerEpoch,
        fetch: impl FnOnce() -> T,
    ) -> T {
        let want = epochs.epoch_for_scalar(self.layer);
        if self.seen_epoch != want {
            self.value = fetch();
            self.seen_epoch = want;
        }
        self.value
    }
}

/// Config: authoritative chunk slab shape for field layers (mirrors first seen [`crate::terrain::generation::ChunkCellMatrix::size`]).
#[derive(Resource, Clone, Copy, Debug)]
pub struct WorldFieldLayerConfig {
    pub cells_per_chunk: UVec2,
}

impl Default for WorldFieldLayerConfig {
    fn default() -> Self {
        Self {
            cells_per_chunk: UVec2::new(32, 32),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_update_class_tiers() {
        assert_eq!(
            FieldScalarLayer::Power.update_class(),
            FieldLayerUpdateClass::Static
        );
        assert_eq!(
            FieldScalarLayer::Control.update_class(),
            FieldLayerUpdateClass::Slow
        );
        assert_eq!(
            FieldScalarLayer::Threat.update_class(),
            FieldLayerUpdateClass::Dynamic
        );
    }

    #[test]
    fn bump_dynamic_leaves_static_consumer_cache() {
        let mut epochs = WorldFieldLayerEpoch::default();
        let mut cache = FieldLayerConsumerCache::new(FieldScalarLayer::Power, 99.0_f32);

        let v0 = cache.value_or_refresh(&epochs, || 1.0);
        assert!((v0 - 1.0).abs() < f32::EPSILON);
        assert_eq!(cache.seen_epoch(), 0);

        epochs.bump(FieldLayerUpdateClass::Dynamic);
        let v1 = cache.value_or_refresh(&epochs, || 2.0);
        assert!((v1 - 1.0).abs() < f32::EPSILON, "static tier unchanged");
        assert_eq!(cache.seen_epoch(), 0);

        epochs.bump(FieldLayerUpdateClass::Static);
        let v2 = cache.value_or_refresh(&epochs, || 3.0);
        assert!((v2 - 3.0).abs() < f32::EPSILON);
        assert_eq!(cache.seen_epoch(), 1);
    }

    #[test]
    fn bump_only_touches_matching_class_epoch() {
        let mut epochs = WorldFieldLayerEpoch::default();
        epochs.bump(FieldLayerUpdateClass::Slow);
        assert_eq!(epochs.static_epoch, 0);
        assert_eq!(epochs.slow_epoch, 1);
        assert_eq!(epochs.dynamic_epoch, 0);
    }

    #[test]
    fn slow_consumer_ignores_dynamic_bump() {
        let mut epochs = WorldFieldLayerEpoch::default();
        let mut cache = FieldLayerConsumerCache::new(FieldScalarLayer::Morale, 0.0_f32);
        let _ = cache.value_or_refresh(&epochs, || 0.5);
        epochs.bump(FieldLayerUpdateClass::Dynamic);
        let v = cache.value_or_refresh(&epochs, || 0.9);
        assert!((v - 0.5).abs() < f32::EPSILON);
    }
}