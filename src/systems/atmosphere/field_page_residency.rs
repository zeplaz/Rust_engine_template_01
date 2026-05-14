//! Virtualized atmosphere field pages — chunk pages → atlas slots → dirty residency.

use std::collections::HashMap;

use bevy::prelude::*;

/// Chunk span covered by one virtual field page (world chunk coords).
pub const ATMOSPHERE_FIELD_CHUNKS_PER_PAGE: i32 = 8;

/// Resident page metadata (CPU residency table).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtmosphereFieldPage {
    pub page_coord: IVec2,
    pub atlas_slot: UVec2,
    pub last_used_frame: u64,
    pub dirty: bool,
}

/// GPU-visible page table row (mirrors WGSL `AtmospherePageEntry`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AtmospherePageEntry {
    pub atlas_origin: UVec2,
    pub valid: u32,
    pub _pad: u32,
}

impl AtmospherePageEntry {
    #[must_use]
    pub const fn invalid() -> Self {
        Self {
            atlas_origin: UVec2::ZERO,
            valid: 0,
            _pad: 0,
        }
    }

    #[must_use]
    pub fn from_page(page: &AtmosphereFieldPage) -> Self {
        Self {
            atlas_origin: page.atlas_slot,
            valid: if page.dirty { 2 } else { 1 },
            _pad: 0,
        }
    }
}

/// CPU page residency manager for the single-GPU-atlas virtualization slice.
#[derive(Resource, Clone, Debug, Default)]
pub struct AtmosphereFieldResidencyTable {
    pub pages: HashMap<IVec2, AtmosphereFieldPage>,
    pub max_resident_pages: usize,
    pub frame: u64,
}

impl AtmosphereFieldResidencyTable {
    #[must_use]
    pub fn with_capacity(max_resident_pages: usize) -> Self {
        Self {
            pages: HashMap::new(),
            max_resident_pages: max_resident_pages.max(1),
            frame: 0,
        }
    }

    pub fn advance_frame(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

    #[must_use]
    pub fn page_coord_for_chunk(chunk: IVec2) -> IVec2 {
        let span = ATMOSPHERE_FIELD_CHUNKS_PER_PAGE.max(1);
        IVec2::new(floor_div(chunk.x, span), floor_div(chunk.y, span))
    }

    pub fn touch_chunk(&mut self, chunk: IVec2) -> &mut AtmosphereFieldPage {
        let page_coord = Self::page_coord_for_chunk(chunk);
        self.touch_page(page_coord)
    }

    pub fn touch_page(&mut self, page_coord: IVec2) -> &mut AtmosphereFieldPage {
        if !self.pages.contains_key(&page_coord) {
            self.evict_lru_if_needed();
            let slot = self.allocate_slot(page_coord);
            self.pages.insert(
                page_coord,
                AtmosphereFieldPage {
                    page_coord,
                    atlas_slot: slot,
                    last_used_frame: self.frame,
                    dirty: true,
                },
            );
        }
        let page = self
            .pages
            .get_mut(&page_coord)
            .expect("page must exist after touch");
        page.last_used_frame = self.frame;
        page
    }

    pub fn mark_dirty_region(&mut self, min: IVec2, max: IVec2) {
        let min_page = Self::page_coord_for_chunk(min);
        let max_page = Self::page_coord_for_chunk(max);
        for y in min_page.y..=max_page.y {
            for x in min_page.x..=max_page.x {
                let page = self.touch_page(IVec2::new(x, y));
                page.dirty = true;
            }
        }
    }

    #[must_use]
    pub fn dirty_resident_pages(&self) -> Vec<AtmosphereFieldPage> {
        let mut pages: Vec<_> = self
            .pages
            .values()
            .copied()
            .filter(|page| page.dirty)
            .collect();
        pages.sort_by_key(|page| (page.page_coord.x, page.page_coord.y));
        pages
    }

    #[must_use]
    pub fn gpu_page_entries(&self) -> Vec<AtmospherePageEntry> {
        let mut pages: Vec<_> = self.pages.values().collect();
        pages.sort_by_key(|page| (page.page_coord.x, page.page_coord.y));
        pages
            .iter()
            .map(|page| AtmospherePageEntry::from_page(page))
            .collect()
    }

    fn allocate_slot(&self, page_coord: IVec2) -> UVec2 {
        let span = ATMOSPHERE_FIELD_CHUNKS_PER_PAGE.max(1);
        let slot = IVec2::new(
            page_coord.x.rem_euclid(span),
            page_coord.y.rem_euclid(span),
        );
        UVec2::new(slot.x as u32, slot.y as u32)
    }

    fn evict_lru_if_needed(&mut self) {
        if self.pages.len() < self.max_resident_pages {
            return;
        }
        let Some(oldest) = self
            .pages
            .values()
            .min_by_key(|page| page.last_used_frame)
            .map(|page| page.page_coord)
        else {
            return;
        };
        self.pages.remove(&oldest);
    }
}

#[must_use]
fn floor_div(value: i32, divisor: i32) -> i32 {
    if value >= 0 {
        value / divisor
    } else {
        (value - divisor + 1) / divisor
    }
}

pub fn sync_atmosphere_field_page_residency(
    fire: Option<Res<crate::render::sim_visual_extract::FireVisualFrame>>,
    mut table: ResMut<AtmosphereFieldResidencyTable>,
) {
    table.advance_frame();
    let Some(fire) = fire else {
        return;
    };
    if fire.chunk_heat.is_empty() {
        return;
    }
    let mut min = fire.chunk_heat[0].chunk;
    let mut max = min;
    for row in &fire.chunk_heat {
        min = IVec2::new(min.x.min(row.chunk.x), min.y.min(row.chunk.y));
        max = IVec2::new(max.x.max(row.chunk.x), max.y.max(row.chunk.y));
        table.touch_chunk(row.chunk);
    }
    table.mark_dirty_region(min, max);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_coord_partitions_chunks_into_fixed_span() {
        assert_eq!(
            AtmosphereFieldResidencyTable::page_coord_for_chunk(IVec2::new(0, 0)),
            IVec2::ZERO
        );
        assert_eq!(
            AtmosphereFieldResidencyTable::page_coord_for_chunk(IVec2::new(7, 7)),
            IVec2::ZERO
        );
        assert_eq!(
            AtmosphereFieldResidencyTable::page_coord_for_chunk(IVec2::new(8, 0)),
            IVec2::new(1, 0)
        );
    }

    #[test]
    fn dirty_region_marks_touched_pages() {
        let mut table = AtmosphereFieldResidencyTable::with_capacity(8);
        table.frame = 1;
        table.mark_dirty_region(IVec2::new(0, 0), IVec2::new(9, 0));
        assert_eq!(table.dirty_resident_pages().len(), 2);
    }

    #[test]
    fn gpu_page_entries_mark_valid_and_dirty() {
        let mut table = AtmosphereFieldResidencyTable::with_capacity(4);
        table.frame = 3;
        let page = table.touch_chunk(IVec2::new(2, 2));
        page.dirty = true;
        let entries = table.gpu_page_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].valid, 2);
    }
}
