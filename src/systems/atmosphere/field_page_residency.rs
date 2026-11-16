//! Virtualized atmosphere field pages — chunk pages → atlas slots → dirty residency.
//!
//! # MEM-10 — virtual memory / page faults / TLB (gea4v1 3.5.2 / 3.5.2.3–4 p186-189;
//! LEGACY `memory_architectures`, gea_v1_e4 §3.5 p183)
//!
//! Plan: PLAN-REF-BOOKS-UPGRADE-v3 slice V4-MEM10.
//!
//! * **Virtual page** — `page_coord` is the virtual address / handle; `atlas_slot` is the
//!   physical frame. Consumers address by `page_coord`, never by a cached slot pointer.
//! * **Page table + residency** — [`AtmosphereFieldResidencyTable::pages`] is the residency
//!   page table (`page_coord` → [`AtmosphereFieldPage`]); capacity is capped by
//!   [`ATMOSPHERE_FIELD_ATLAS_SLOTS`] physical frames.
//! * **Fault-in (3.5.2.3)** — [`AtmosphereFieldResidencyTable::touch_page`] of a missing
//!   coord allocates a slot, inserts the row, and marks `dirty` (re-upload).
//! * **TLB (3.5.2.4)** — [`AtmosphereFieldPageTable`] / [`AtmosphereFieldResidencyTable::gpu_page_entries`]
//!   rebuild the GPU translation rows from the residency table each sync; do not keep a
//!   stale `atlas_origin` outside that rebuild.
//!
//! # MEM-04 — fragmentation + relocation (gea4v1 6.2.2 / 6.2.2.1–2; LEGACY `memory_management`, gea_v1_e4 p412)
//!
//! * **Fixed-size pages sidestep fragmentation (6.2.2.1).** One page always spans
//!   [`ATMOSPHERE_FIELD_CHUNKS_PER_PAGE`]² chunks and occupies exactly one atlas slot out of
//!   [`ATMOSPHERE_FIELD_ATLAS_SLOTS`]; slots are uniform, so evicting any page frees a slot that fits
//!   any other page (no external fragmentation, nothing to compact).
//!   **Do not regress** the fixed page-size / slot-pool contracts below.
//! * **Relocation needs handle indirection (6.2.2.2).** A page's identity is its virtual
//!   `page_coord` (the handle). A page that is evicted and faulted back in may land in a *different*
//!   atlas slot; nothing may cache `atlas_slot`/`atlas_origin`. Readers resolve `page_coord` →
//!   [`AtmosphereFieldPage`] every frame, and the GPU reads the per-frame rebuilt
//!   [`AtmosphereFieldPageTable`] rows ([`AtmospherePageEntry`]), never a raw slot address.
//!   Fault-in always marks the page dirty so its contents are re-uploaded into the new slot.

use std::collections::HashMap;

use bevy::prelude::*;

/// Chunk span covered by one virtual field page (world chunk coords).
pub const ATMOSPHERE_FIELD_CHUNKS_PER_PAGE: i32 = 8;

/// Physical atlas slots (pool size): one slot per page, `PAGE_SPAN × PAGE_SPAN`.
pub const ATMOSPHERE_FIELD_ATLAS_SLOTS: usize =
    (ATMOSPHERE_FIELD_CHUNKS_PER_PAGE * ATMOSPHERE_FIELD_CHUNKS_PER_PAGE) as usize;

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
#[derive(Resource, Clone, Debug)]
pub struct AtmosphereFieldResidencyTable {
    pub pages: HashMap<IVec2, AtmosphereFieldPage>,
    pub max_resident_pages: usize,
    pub frame: u64,
}

impl Default for AtmosphereFieldResidencyTable {
    /// Full pool: every atlas slot may be resident. (A derived `Default` gave capacity 0, which
    /// evicted down to a single resident page and starved the dirty-region filter.)
    fn default() -> Self {
        Self::with_capacity(ATMOSPHERE_FIELD_ATLAS_SLOTS)
    }
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

    /// Residency limit actually enforced: never more pages than physical atlas slots.
    #[must_use]
    pub fn effective_capacity(&self) -> usize {
        self.max_resident_pages.clamp(1, ATMOSPHERE_FIELD_ATLAS_SLOTS)
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

    /// Lowest free atlas slot (row-major). Deterministic and unique per resident page — the slot
    /// is *not* a function of `page_coord`, so a re-faulted page may relocate.
    fn allocate_slot(&self, _page_coord: IVec2) -> UVec2 {
        let span = ATMOSPHERE_FIELD_CHUNKS_PER_PAGE.max(1) as u32;
        let mut used = [false; ATMOSPHERE_FIELD_ATLAS_SLOTS];
        for page in self.pages.values() {
            let idx = (page.atlas_slot.y * span + page.atlas_slot.x) as usize;
            if let Some(flag) = used.get_mut(idx) {
                *flag = true;
            }
        }
        let free = used.iter().position(|taken| !taken).unwrap_or(0) as u32;
        UVec2::new(free % span, free / span)
    }

    fn evict_lru_if_needed(&mut self) {
        if self.pages.len() < self.effective_capacity() {
            return;
        }
        let Some(oldest) = self
            .pages
            .values()
            .min_by_key(|page| (page.last_used_frame, page.page_coord.x, page.page_coord.y))
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

/// GPU page table mirror for partial compute dispatch (resident dirty pages).
#[derive(Resource, Clone, Debug, Default)]
pub struct AtmosphereFieldPageTable {
    pub entries: Vec<AtmospherePageEntry>,
}

pub fn sync_atmosphere_field_page_table(
    table: Res<AtmosphereFieldResidencyTable>,
    mut gpu_table: ResMut<AtmosphereFieldPageTable>,
) {
    gpu_table.entries = table.gpu_page_entries();
}

pub fn sync_atmosphere_field_page_residency(
    fire: Option<Res<crate::render::FireSimulationSnapshot>>,
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
    fn page_table_tracks_resident_gpu_entries() {
        let mut table = AtmosphereFieldResidencyTable::with_capacity(4);
        table.touch_chunk(IVec2::new(2, 2));
        let gpu = AtmosphereFieldPageTable {
            entries: table.gpu_page_entries(),
        };
        assert_eq!(gpu.entries.len(), 1);
        assert_ne!(gpu.entries[0].valid, 0);
    }

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

    /// MEM-04 tripwire: page geometry is a fixed pool — span, slot count and GPU row size are pinned.
    #[test]
    fn mem04_page_size_is_fixed() {
        assert_eq!(ATMOSPHERE_FIELD_CHUNKS_PER_PAGE, 8);
        assert_eq!(ATMOSPHERE_FIELD_ATLAS_SLOTS, 64);
        assert_eq!(std::mem::size_of::<AtmospherePageEntry>(), 16);
        let table = AtmosphereFieldResidencyTable::default();
        assert_eq!(table.effective_capacity(), ATMOSPHERE_FIELD_ATLAS_SLOTS);
        assert_eq!(
            AtmosphereFieldResidencyTable::with_capacity(10_000).effective_capacity(),
            ATMOSPHERE_FIELD_ATLAS_SLOTS
        );
        assert_eq!(AtmosphereFieldResidencyTable::with_capacity(0).effective_capacity(), 1);
    }

    /// MEM-04 tripwire: resident pages never share an atlas slot, even when their `page_coord`s
    /// collide modulo the span — slots are pool-allocated, not derived from the coord.
    #[test]
    fn mem04_resident_pages_get_unique_atlas_slots() {
        let mut table = AtmosphereFieldResidencyTable::default();
        for i in 0..ATMOSPHERE_FIELD_ATLAS_SLOTS as i32 {
            table.touch_page(IVec2::new(i * ATMOSPHERE_FIELD_CHUNKS_PER_PAGE, 0));
        }
        assert_eq!(table.pages.len(), ATMOSPHERE_FIELD_ATLAS_SLOTS);
        let mut slots: Vec<_> = table
            .pages
            .values()
            .map(|p| (p.atlas_slot.x, p.atlas_slot.y))
            .collect();
        slots.sort_unstable();
        slots.dedup();
        assert_eq!(slots.len(), ATMOSPHERE_FIELD_ATLAS_SLOTS, "slot aliasing");
    }

    /// MEM-04 tripwire: eviction + fault-in is handle-addressed. A re-faulted page is found by
    /// `page_coord`, may land in a different slot, is dirty (re-upload), and its GPU row is rebuilt
    /// from the table rather than from a cached slot.
    #[test]
    fn mem04_fault_in_resolves_by_handle_and_relocates_slot() {
        let mut table = AtmosphereFieldResidencyTable::with_capacity(2);
        let (a, b, c) = (IVec2::new(0, 0), IVec2::new(1, 0), IVec2::new(2, 0));
        table.frame = 1;
        let slot_a = table.touch_page(a).atlas_slot;
        table.frame = 2;
        let slot_b0 = table.touch_page(b).atlas_slot;
        assert_ne!(slot_a, slot_b0);
        table.frame = 3;
        table.touch_page(a); // b is LRU
        table.frame = 4;
        let slot_c = table.touch_page(c).atlas_slot; // evicts b, recycles its slot
        assert!(!table.pages.contains_key(&b));
        assert_eq!(slot_c, slot_b0, "freed slot is recycled (no hole)");
        table.frame = 5;
        table.touch_page(c); // a is LRU
        table.frame = 6;
        let page_b = *table.touch_page(b); // fault b back in: evicts a, takes a's slot
        assert!(!table.pages.contains_key(&a));
        assert_eq!(page_b.atlas_slot, slot_a);
        assert_ne!(page_b.atlas_slot, slot_b0, "b relocated; only the handle is stable");
        assert!(page_b.dirty, "fault-in must mark dirty for re-upload");
        let rows = table.gpu_page_entries();
        assert_eq!(rows.len(), 2);
        let row_b = rows
            .iter()
            .find(|r| r.atlas_origin == page_b.atlas_slot)
            .expect("row resolves via table slot");
        assert_eq!(row_b.valid, 2);
        let uniq: std::collections::HashSet<_> =
            rows.iter().map(|r| (r.atlas_origin.x, r.atlas_origin.y)).collect();
        assert_eq!(uniq.len(), rows.len());
    }

    /// MEM-04 tripwire: LRU eviction is deterministic on ties (frame, then coord) — no HashMap order.
    #[test]
    fn mem04_lru_tie_break_is_deterministic() {
        let mut table = AtmosphereFieldResidencyTable::with_capacity(2);
        table.frame = 1;
        table.touch_page(IVec2::new(5, 5));
        table.touch_page(IVec2::new(-1, 3));
        table.touch_page(IVec2::new(9, 9)); // tie on frame 1 -> evicts smallest (x, y) = (-1, 3)
        assert!(!table.pages.contains_key(&IVec2::new(-1, 3)));
        assert!(table.pages.contains_key(&IVec2::new(5, 5)));
    }

    /// MEM-10 tripwire: virtual handle (`page_coord`) is identity; physical `atlas_slot` is not.
    #[test]
    fn mem10_page_coord_is_virtual_handle_not_physical_slot() {
        let mut table = AtmosphereFieldResidencyTable::with_capacity(4);
        let va = IVec2::new(3, -1);
        let page = *table.touch_page(va);
        assert_eq!(page.page_coord, va);
        assert!(
            table.pages.contains_key(&va),
            "residency page table keyed by virtual coord"
        );
        // Physical slot is pool-assigned — not equal to the virtual coord components.
        assert!(
            page.atlas_slot.x < ATMOSPHERE_FIELD_CHUNKS_PER_PAGE as u32
                && page.atlas_slot.y < ATMOSPHERE_FIELD_CHUNKS_PER_PAGE as u32
        );
        let resolved = table.pages.get(&va).expect("resolve by handle");
        assert_eq!(resolved.atlas_slot, page.atlas_slot);
    }

    /// MEM-10 tripwire: fault-in of a missing page allocates a table row and marks dirty.
    #[test]
    fn mem10_fault_in_allocates_page_table_row() {
        let mut table = AtmosphereFieldResidencyTable::with_capacity(4);
        let va = IVec2::new(2, 2);
        assert!(!table.pages.contains_key(&va));
        table.frame = 7;
        let page = *table.touch_page(va);
        assert_eq!(page.page_coord, va);
        assert_eq!(page.last_used_frame, 7);
        assert!(page.dirty, "fault-in must dirty for re-upload");
        assert_eq!(table.pages.len(), 1);
        let entries = table.gpu_page_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].atlas_origin, page.atlas_slot);
        assert_eq!(entries[0].valid, 2);
    }

    /// MEM-10 tripwire: GPU TLB rows come from the residency table rebuild, not a cached slot ptr.
    #[test]
    fn mem10_tlb_rows_rebuild_from_residency_table() {
        let mut table = AtmosphereFieldResidencyTable::with_capacity(2);
        let (a, b, c) = (IVec2::new(0, 0), IVec2::new(1, 0), IVec2::new(2, 0));
        table.frame = 1;
        table.touch_page(a);
        table.frame = 2;
        table.touch_page(b);
        table.frame = 3;
        table.touch_page(a); // b LRU
        table.frame = 4;
        table.touch_page(c); // evicts b
        assert!(!table.pages.contains_key(&b));
        // TLB = rebuilt entries: every row's atlas_origin matches a live residency page.
        let rows = table.gpu_page_entries();
        assert_eq!(rows.len(), table.pages.len());
        for row in &rows {
            assert_ne!(row.valid, 0);
            assert!(
                table
                    .pages
                    .values()
                    .any(|p| p.atlas_slot == row.atlas_origin),
                "TLB origin must resolve through residency table"
            );
        }
    }

    /// MEM-10 tripwire: MEM-04 fixed page-size contract stays pinned (no regress).
    #[test]
    fn mem10_does_not_regress_fixed_page_size() {
        assert_eq!(ATMOSPHERE_FIELD_CHUNKS_PER_PAGE, 8);
        assert_eq!(ATMOSPHERE_FIELD_ATLAS_SLOTS, 64);
        assert_eq!(std::mem::size_of::<AtmospherePageEntry>(), 16);
    }
}
