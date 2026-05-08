//! D8 flow direction, priority-flood–style depression fill, accumulation, river/lake masks.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::HydrologyParams;

/// One connected lake / depression region (local tile coordinates).
#[derive(Clone, Debug, Default)]
pub struct LakeRegion {
    pub cells: Vec<(u32, u32)>,
}

/// Raster hydrology products for a `width × height` grid (row-major `dem`).
#[derive(Clone, Debug)]
pub struct HydrologyResult {
    pub rivers: Vec<Vec<(u32, u32)>>,
    pub lakes: Vec<LakeRegion>,
    /// Upstream area count per cell (includes self).
    pub accumulation: Vec<f32>,
    pub river_mask: Vec<bool>,
    pub lake_mask: Vec<bool>,
    /// Filled DEM after depression processing (for debugging / downstream use).
    pub filled_dem: Vec<f32>,
}

#[inline]
fn idx(w: usize, x: u32, y: u32) -> usize {
    y as usize * w + x as usize
}

/// Neighbor offsets — D8, order is deterministic for tie-breaking.
const D8: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OrdF32(u32);

impl OrdF32 {
    #[inline]
    fn from_f32(v: f32) -> Self {
        Self(v.to_bits())
    }
}

impl PartialOrd for OrdF32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrdF32 {
    fn cmp(&self, other: &Self) -> Ordering {
        f32::from_bits(self.0).total_cmp(&f32::from_bits(other.0))
    }
}

/// Barnes-style priority flood: raises interior pits so drainage reaches the border.
fn priority_flood_fill(dem: &[f32], w: usize, h: usize) -> Vec<f32> {
    let n = w * h;
    let mut filled = dem.to_vec();
    let mut visited = vec![false; n];
    let mut heap: BinaryHeap<std::cmp::Reverse<(OrdF32, usize)>> = BinaryHeap::new();

    let push_border = |x: usize, y: usize, heap: &mut BinaryHeap<_>, visited: &mut [bool]| {
        let i = y * w + x;
        if !visited[i] {
            visited[i] = true;
            heap.push(std::cmp::Reverse((OrdF32::from_f32(dem[i]), i)));
        }
    };

    if w == 0 || h == 0 {
        return filled;
    }
    if w == 1 && h == 1 {
        return filled;
    }

    for x in 0..w {
        push_border(x, 0, &mut heap, &mut visited);
        push_border(x, h - 1, &mut heap, &mut visited);
    }
    for y in 0..h {
        push_border(0, y, &mut heap, &mut visited);
        push_border(w - 1, y, &mut heap, &mut visited);
    }

    while let Some(std::cmp::Reverse((_, ci))) = heap.pop() {
        let cx = ci % w;
        let cy = ci / w;
        for (dx, dy) in D8 {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                continue;
            }
            let nx = nx as usize;
            let ny = ny as usize;
            let ni = ny * w + nx;
            if !visited[ni] {
                visited[ni] = true;
                filled[ni] = filled[ni].max(filled[ci]).max(dem[ni]);
                heap.push(std::cmp::Reverse((OrdF32::from_f32(filled[ni]), ni)));
            }
        }
    }

    filled
}

fn d8_downstream(filled: &[f32], w: usize, h: usize) -> Vec<Option<usize>> {
    let mut down: Vec<Option<usize>> = vec![None; w * h];
    for y in 0..h as u32 {
        for x in 0..w as u32 {
            let i = idx(w, x, y);
            let z = filled[i];
            let mut best_z = z;
            let mut best_j: Option<usize> = None;
            for (dx, dy) in D8 {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let j = idx(w, nx as u32, ny as u32);
                let nz = filled[j];
                match nz.total_cmp(&best_z) {
                    Ordering::Less => {
                        best_z = nz;
                        best_j = Some(j);
                    }
                    Ordering::Equal => {}
                    Ordering::Greater => {}
                }
            }
            if best_j.is_some_and(|j| filled[j] < z) {
                down[i] = best_j;
            }
        }
    }
    down
}

/// Per-cell runoff input into flow accumulation. Uniform when `moisture` is `None` or wrong length.
/// Wet cells (high moisture / rainfall proxy) inject more water, shaping channel hierarchy.
fn accumulation_inputs(moisture: Option<&[f32]>, n: usize) -> Vec<f32> {
    let uniform = |i: usize| {
        moisture
            .and_then(|m| m.get(i).copied())
            .map(|mv| {
                let m = mv.clamp(0.0, 1.0);
                // Dry ~0.12, wet ~1.35 — keeps arid Interiors from dominating catchments.
                0.12 + m * 1.23
            })
            .unwrap_or(1.0)
    };
    (0..n).map(uniform).collect()
}

fn accumulate(
    downstream: &[Option<usize>],
    filled: &[f32],
    w: usize,
    h: usize,
    moisture: Option<&[f32]>,
) -> Vec<f32> {
    let n = w * h;
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| filled[b].total_cmp(&filled[a]));

    let mut acc = accumulation_inputs(moisture, n);
    for &i in &order {
        if let Some(ds) = downstream[i] {
            acc[ds] += acc[i];
        }
    }
    acc
}

fn max_slice(s: &[f32]) -> f32 {
    s.iter().copied().fold(0.0f32, f32::max)
}

fn trace_downstream_path(
    start: usize,
    river_mask: &[bool],
    down: &[Option<usize>],
    w: usize,
    _h: usize,
    max_len: usize,
) -> Vec<(u32, u32)> {
    let mut path = Vec::new();
    let mut cur = Some(start);
    let mut guard = 0usize;
    while let Some(i) = cur {
        if guard >= max_len {
            break;
        }
        guard += 1;
        let x = (i % w) as u32;
        let y = (i / w) as u32;
        path.push((x, y));
        if !river_mask.get(i).copied().unwrap_or(false) && path.len() > 1 {
            break;
        }
        cur = down[i];
    }
    path
}

fn extract_river_traces(
    w: usize,
    h: usize,
    acc: &[f32],
    river_mask: &[bool],
    down: &[Option<usize>],
    river_traces: u32,
) -> Vec<Vec<(u32, u32)>> {
    if river_traces == 0 {
        return Vec::new();
    }
    let n = w * h;
    let mut seeds: Vec<(usize, f32)> = river_mask
        .iter()
        .enumerate()
        .filter_map(|(i, &m)| m.then_some((i, acc[i])))
        .collect();
    seeds.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));

    let mut used = vec![false; n];
    let mut paths = Vec::new();
    for (i, _) in seeds {
        if paths.len() >= river_traces as usize {
            break;
        }
        if used[i] {
            continue;
        }
        let path = trace_downstream_path(i, river_mask, down, w, h, w * h * 2);
        if path.len() < 2 {
            continue;
        }
        for &(x, y) in &path {
            used[idx(w, x, y)] = true;
        }
        paths.push(path);
    }
    paths
}

fn label_lakes(
    w: usize,
    h: usize,
    _dem: &[f32],
    _filled: &[f32],
    lake_mask: &[bool],
) -> Vec<LakeRegion> {
    let n = w * h;
    let mut visited = vec![false; n];
    let mut regions = Vec::new();
    for y in 0..h as u32 {
        for x in 0..w as u32 {
            let i = idx(w, x, y);
            if visited[i] || !lake_mask[i] {
                continue;
            }
            let mut stack = vec![i];
            let mut cells = Vec::new();
            visited[i] = true;
            while let Some(ci) = stack.pop() {
                let cx = ci % w;
                let cy = ci / w;
                cells.push((cx as u32, cy as u32));
                for (dx, dy) in D8 {
                    let nx = cx as i32 + dx;
                    let ny = cy as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let ni = ny as usize * w + nx as usize;
                    if visited[ni] || !lake_mask[ni] {
                        continue;
                    }
                    visited[ni] = true;
                    stack.push(ni);
                }
            }
            if !cells.is_empty() {
                regions.push(LakeRegion { cells });
            }
        }
    }
    regions
}

/// Full hydrology for a single rectangle (`dem.len() == width * height`).
///
/// When `moisture` matches the grid length, it scales per-cell runoff into accumulation (rainfall proxy).
pub fn compute_hydrology_rect(
    width: u32,
    height: u32,
    dem: &[f32],
    params: &HydrologyParams,
    river_traces: u32,
    moisture: Option<&[f32]>,
) -> HydrologyResult {
    let w = width as usize;
    let h = height as usize;
    let n = w.saturating_mul(h);
    if n == 0 || dem.len() != n {
        return HydrologyResult {
            rivers: Vec::new(),
            lakes: Vec::new(),
            accumulation: Vec::new(),
            river_mask: Vec::new(),
            lake_mask: Vec::new(),
            filled_dem: dem.to_vec(),
        };
    }

    let moisture = moisture.filter(|m| m.len() == n);

    let filled = priority_flood_fill(dem, w, h);
    let down = d8_downstream(&filled, w, h);
    let acc = accumulate(&down, &filled, w, h, moisture);
    let mx = max_slice(&acc).max(1.0);
    let acc_th_by_max = params.river_acc_quantile * mx;
    let q = params.river_acc_quantile.clamp(0.0, 1.0);
    let acc_th_by_rank = if n <= 1 {
        acc[0]
    } else {
        let mut sorted = acc.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let idx = ((1.0 - q) * (n - 1) as f32).round() as usize;
        sorted[idx.min(n - 1)]
    };
    // More permissive of the two: avoids “one huge sink” making 0.12×max untouchable for typical cells.
    let acc_th = acc_th_by_max.min(acc_th_by_rank);

    let mut river_mask = vec![false; n];
    for i in 0..n {
        if acc[i] >= acc_th {
            river_mask[i] = true;
        }
    }

    let mut lake_mask = vec![false; n];
    for i in 0..n {
        let is_dep = filled[i] > dem[i] + 1e-6;
        let is_low = dem[i] <= params.water_line;
        if is_dep || is_low {
            lake_mask[i] = true;
        }
    }
    for i in 0..n {
        if river_mask[i] {
            lake_mask[i] = false;
        }
    }

    let rivers = extract_river_traces(w, h, &acc, &river_mask, &down, river_traces);
    let lakes = label_lakes(w, h, dem, &filled, &lake_mask);

    HydrologyResult {
        rivers,
        lakes,
        accumulation: acc,
        river_mask,
        lake_mask,
        filled_dem: filled,
    }
}

/// World-map hydrology using full `height_grid` and `WorldGenParams` feature counts.
///
/// `lake_count` limits how many lake regions are returned (largest by cell count first); `0` clears
/// the `lakes` list (river / lake masks and accumulation are unchanged).
pub fn compute_hydrology_world(
    width: u32,
    height: u32,
    height_grid: &[f32],
    moisture_grid: Option<&[f32]>,
    params: &HydrologyParams,
    river_count: u32,
    lake_count: u32,
) -> HydrologyResult {
    let mut r = compute_hydrology_rect(
        width,
        height,
        height_grid,
        params,
        river_count,
        moisture_grid,
    );
    if lake_count == 0 {
        r.lakes.clear();
    } else if r.lakes.len() > lake_count as usize {
        r.lakes.sort_by(|a, b| b.cells.len().cmp(&a.cells.len()));
        r.lakes.truncate(lake_count as usize);
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plane_dem(w: u32, h: u32) -> Vec<f32> {
        let mut v = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let fx = x as f32 / (w.saturating_sub(1).max(1) as f32);
                let fy = y as f32 / (h.saturating_sub(1).max(1) as f32);
                v.push(fx * 0.6 + fy * 0.4);
            }
        }
        v
    }

    #[test]
    fn priority_flood_raises_enclosed_pit() {
        let w = 5usize;
        let h = 5usize;
        let mut dem = vec![1.0f32; w * h];
        dem[2 * w + 2] = 0.0;
        let filled = priority_flood_fill(&dem, w, h);
        assert!(
            (filled[2 * w + 2] - 1.0).abs() < 1e-4,
            "center pit should spill up to rim height"
        );
    }

    #[test]
    fn flow_accumulation_deterministic() {
        let w = 8u32;
        let h = 8u32;
        let dem = plane_dem(w, h);
        let p = HydrologyParams::default();
        let a = compute_hydrology_rect(w, h, &dem, &p, 2, None);
        let b = compute_hydrology_rect(w, h, &dem, &p, 2, None);
        assert_eq!(a.accumulation, b.accumulation);
        assert_eq!(a.river_mask, b.river_mask);
        assert_eq!(a.lake_mask, b.lake_mask);
    }

    #[test]
    fn priority_flood_no_pits() {
        let w = 6u32;
        let h = 6u32;
        let dem = plane_dem(w, h);
        let filled = priority_flood_fill(&dem, w as usize, h as usize);
        for i in 0..dem.len() {
            assert!(
                (filled[i] - dem[i]).abs() < 1e-4,
                "planar dem should not change materially at {}",
                i
            );
        }
    }

    #[test]
    fn river_threshold_count() {
        let w = 12u32;
        let h = 12u32;
        let dem = plane_dem(w, h);
        let p = HydrologyParams::default();
        let r = compute_hydrology_rect(w, h, &dem, &p, 3, None);
        let river_cells = r.river_mask.iter().filter(|x| **x).count();
        assert!(river_cells > 0, "expected some high-accumulation channels");
        assert!(r.rivers.len() <= 3);
    }
}
