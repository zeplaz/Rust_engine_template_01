"""SUBPROGRAM pixel_conclude — one shot, then return.

Fortran-style unit: subroutines write, functions return values.
No daemon, no heartbeat, no agent loop.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

# Fixed clustering contract. Same image + these constants => same labels.
KMEANS_K = 4
KMEANS_SEED = 17
KMEANS_MAX_ITER = 24
HIST_BIN = 16
HIST_BIN_COARSE = 32
HIST_OCCUPIED_CAP = 512
MEANSHIFT_BANDWIDTH = 48.0
MEANSHIFT_MAX_ITER = 16
LOD_FILL_MIN = 0.85
LOD_EACH_FRAC_MIN = 0.15
LOD_COMBINED_FRAC_MIN = 0.40
DEBUG_FILL_MIN = 0.90
DEBUG_AREA_FRAC_MIN = 0.05

KINDS = ("gui", "ui", "world", "art")

_LABEL_PALETTE = (
    (31, 119, 180),
    (255, 127, 14),
    (44, 160, 44),
    (214, 39, 40),
    (148, 103, 189),
    (140, 86, 75),
    (227, 119, 194),
    (127, 127, 127),
)


def ingest(image_path: str | Path) -> dict:
    """SUBROUTINE ingest — load a PNG. Missing or unreadable files fail closed."""
    path = Path(image_path)
    if not path.is_file():
        raise FileNotFoundError(f"image missing: {path}")
    if path.suffix.lower() != ".png":
        raise ValueError(f"png required: {path}")
    try:
        from PIL import Image
    except ImportError as exc:
        raise RuntimeError("Pillow is required to ingest a PNG") from exc
    try:
        with Image.open(path) as im:
            im.load()
            rgb = im.convert("RGB")
            width, height = rgb.size
            raw = rgb.tobytes()
    except (FileNotFoundError, ValueError, RuntimeError):
        raise
    except Exception as exc:
        raise ValueError(f"png unreadable: {path}") from exc
    if width <= 0 or height <= 0 or len(raw) != width * height * 3:
        raise ValueError(f"png has no pixels: {path}")
    pixels = [(raw[i], raw[i + 1], raw[i + 2]) for i in range(0, len(raw), 3)]
    return {
        "path": path,
        "width": width,
        "height": height,
        "pixels": pixels,
        "raw": raw,
    }


def segment_kmeans(
    width: int,
    height: int,
    pixels: list[tuple[int, int, int]],
    k: int = KMEANS_K,
    seed: int = KMEANS_SEED,
    max_iter: int = KMEANS_MAX_ITER,
) -> dict:
    """FUNCTION segment_kmeans — fixed seed, fixed k, no unseeded draw."""
    bins, bin_w = _histogram(pixels)
    keys = sorted(bins)
    if not keys:
        return _empty_seg("kmeans", width, height, pixels, k=0, seed=seed)
    kk = min(int(k), len(keys))
    init_idx = _lcg_indices(int(seed), kk, len(keys))
    centroids = [list(_bin_center(keys[i], bin_w)) for i in init_idx]
    key_label = {key: 0 for key in keys}
    for _ in range(max_iter):
        changed = False
        sums = [[0.0, 0.0, 0.0, 0.0] for _ in centroids]
        for key in keys:
            center = _bin_center(key, bin_w)
            lab = _nearest(center, centroids)
            if key_label[key] != lab:
                changed = True
                key_label[key] = lab
            weight = bins[key]
            sums[lab][0] += center[0] * weight
            sums[lab][1] += center[1] * weight
            sums[lab][2] += center[2] * weight
            sums[lab][3] += weight
        for j, acc in enumerate(sums):
            if acc[3] > 0.0:
                centroids[j] = [acc[0] / acc[3], acc[1] / acc[3], acc[2] / acc[3]]
        if not changed:
            break
    labels = [_key_label(px, bin_w, key_label) for px in pixels]
    return {
        "algorithm": "kmeans",
        "k": kk,
        "seed": int(seed),
        "bin": bin_w,
        "labels": labels,
        "clusters": _scan_clusters(width, height, pixels, labels),
    }


def segment_meanshift(
    width: int,
    height: int,
    pixels: list[tuple[int, int, int]],
    bandwidth: float = MEANSHIFT_BANDWIDTH,
    max_iter: int = MEANSHIFT_MAX_ITER,
) -> dict:
    """FUNCTION segment_meanshift — histogram mode seek. No unseeded draw.

    Occupied color bins are the only samples. A seed is skipped when it
    already sits inside an accepted mode, so the walk is a pure function
    of bin order and the fixed bandwidth.
    """
    bins, bin_w = _histogram(pixels)
    points = [(key, _bin_center(key, bin_w), bins[key]) for key in sorted(bins)]
    if not points:
        return _empty_seg("meanshift", width, height, pixels, bandwidth=float(bandwidth))
    modes: list[tuple[float, float, float]] = []
    for _key, center, _weight in points:
        if any(_dist(center, mode) <= bandwidth for mode in modes):
            continue
        mode = _shift_mode(points, center, bandwidth, max_iter)
        if not any(_dist(mode, kept) <= bandwidth * 0.5 for kept in modes):
            modes.append(mode)
    if not modes:
        modes.append(points[0][1])
    key_label = {key: _nearest(center, modes) for key, center, _weight in points}
    labels = [_key_label(px, bin_w, key_label) for px in pixels]
    return {
        "algorithm": "meanshift",
        "bandwidth": float(bandwidth),
        "bin": bin_w,
        "labels": labels,
        "clusters": _scan_clusters(width, height, pixels, labels),
    }


def write_segmentation_map(
    path: str | Path,
    width: int,
    height: int,
    labels: list[int],
) -> None:
    """SUBROUTINE write_segmentation_map — label PNG from k-means labels."""
    if len(labels) != width * height:
        raise ValueError("label count does not match image size")
    from PIL import Image

    raw = bytearray(width * height * 3)
    n = len(_LABEL_PALETTE)
    for i, lab in enumerate(labels):
        color = _LABEL_PALETTE[int(lab) % n]
        o = i * 3
        raw[o] = color[0]
        raw[o + 1] = color[1]
        raw[o + 2] = color[2]
    dest = Path(path)
    dest.parent.mkdir(parents=True, exist_ok=True)
    Image.frombytes("RGB", (width, height), bytes(raw)).save(dest, format="PNG")


def write_matrix_json(path: str | Path, body: dict) -> None:
    """SUBROUTINE write_matrix_json — counts, centroids, bboxes, kind, sha256."""
    dest = Path(path)
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(json.dumps(body, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def verdict(
    kind: str,
    width: int,
    height: int,
    kmeans_clusters: list[dict],
    meanshift_clusters: list[dict],
    matrix_written: bool,
) -> dict:
    """FUNCTION verdict. Green is returned only when the matrix was written."""
    if kind not in KINDS:
        raise ValueError(f"kind must be one of {', '.join(KINDS)}")
    reasons: list[str] = []
    status = "green"
    sources = (("kmeans", kmeans_clusters), ("meanshift", meanshift_clusters))
    if kind in ("world", "art"):
        for name, clusters in sources:
            if _lod_chrome_hit(clusters, width, height):
                reasons.append(f"dominant_yellow_green_lod_chrome:{name}")
                status = "reject"
    else:
        for name, clusters in sources:
            if _overflow_hit(clusters, width, height):
                reasons.append(f"overflow_or_clipped_debug_block:{name}")
                status = "flag"
    if status == "green" and not matrix_written:
        status = "incomplete"
        reasons.append("matrix_not_written")
    claimed_green = status == "green"
    return {
        "verdict": status,
        "reasons": reasons,
        "honest": (not claimed_green) or bool(matrix_written),
    }


def pixel_conclude(
    image_path: str | Path,
    kind: str,
    out_dir: str | Path | None = None,
) -> dict:
    """SUBPROGRAM pixel_conclude(image_path, kind). Writes artifacts, returns JSON-ready struct."""
    if kind not in KINDS:
        raise ValueError(f"kind must be one of {', '.join(KINDS)}")
    frame = ingest(image_path)
    kmeans = segment_kmeans(frame["width"], frame["height"], frame["pixels"])
    meanshift = segment_meanshift(frame["width"], frame["height"], frame["pixels"])
    out = Path(out_dir) if out_dir is not None else frame["path"].parent / f"{frame['path'].stem}_pixel_conclude"
    out.mkdir(parents=True, exist_ok=True)
    map_path = out / "segmentation_map.png"
    matrix_path = out / f"pixel_matrix_{kind}.json"
    write_segmentation_map(map_path, frame["width"], frame["height"], kmeans["labels"])
    digest = hashlib.sha256(frame["raw"]).hexdigest()
    matrix = {
        "schema": "pixel_conclude_matrix_v1",
        "kind": kind,
        "width": frame["width"],
        "height": frame["height"],
        "sha256": digest,
        "kmeans": _public_segmentation(kmeans),
        "meanshift": _public_segmentation(meanshift),
    }
    write_matrix_json(matrix_path, matrix)
    matrix_written = matrix_path.is_file() and matrix_path.stat().st_size > 0
    judged = verdict(
        kind,
        frame["width"],
        frame["height"],
        kmeans["clusters"],
        meanshift["clusters"],
        matrix_written,
    )
    matrix["verdict"] = judged["verdict"]
    matrix["honest"] = judged["honest"]
    matrix["reasons"] = judged["reasons"]
    matrix["status"] = judged["verdict"]
    if matrix_written:
        write_matrix_json(matrix_path, matrix)
    return {
        "ok": judged["verdict"] == "green",
        "kind": kind,
        "verdict": judged["verdict"],
        "reasons": judged["reasons"],
        "honest": judged["honest"],
        "sha256": digest,
        "width": frame["width"],
        "height": frame["height"],
        "segmentation_map": str(map_path),
        "matrix": str(matrix_path),
        "matrix_written": matrix_written,
        "kmeans": matrix["kmeans"],
        "meanshift": matrix["meanshift"],
        "rules_check": {
            "passed": True,
            "blocked_by": [],
            "seed": str(KMEANS_SEED),
        },
    }


def _histogram(pixels: list[tuple[int, int, int]]) -> tuple[dict[tuple[int, int, int], int], int]:
    bins = _count_bins(pixels, HIST_BIN)
    if len(bins) > HIST_OCCUPIED_CAP:
        return _count_bins(pixels, HIST_BIN_COARSE), HIST_BIN_COARSE
    return bins, HIST_BIN


def _count_bins(pixels: list[tuple[int, int, int]], bin_w: int) -> dict[tuple[int, int, int], int]:
    bins: dict[tuple[int, int, int], int] = {}
    for px in pixels:
        key = (px[0] // bin_w, px[1] // bin_w, px[2] // bin_w)
        bins[key] = bins.get(key, 0) + 1
    return bins


def _bin_center(key: tuple[int, int, int], bin_w: int) -> tuple[float, float, float]:
    half = bin_w / 2.0
    return (key[0] * bin_w + half, key[1] * bin_w + half, key[2] * bin_w + half)


def _key_label(
    pixel: tuple[int, int, int],
    bin_w: int,
    key_label: dict[tuple[int, int, int], int],
) -> int:
    key = (pixel[0] // bin_w, pixel[1] // bin_w, pixel[2] // bin_w)
    return key_label[key]


def _lcg_indices(seed: int, count: int, modulus: int) -> list[int]:
    """Deterministic index pick. Fixed multiplier, no runtime entropy."""
    if count <= 0 or modulus <= 0:
        return []
    x = seed & 0xFFFFFFFF
    if x == 0:
        x = 1
    out: list[int] = []
    seen: set[int] = set()
    guard = 0
    while len(out) < count and guard < count * 16 + 8:
        x = (1664525 * x + 1013904223) & 0xFFFFFFFF
        idx = x % modulus
        if idx not in seen:
            seen.add(idx)
            out.append(idx)
        guard += 1
    cursor = 0
    while len(out) < count:
        if cursor not in seen:
            seen.add(cursor)
            out.append(cursor)
        cursor += 1
    return out


def _nearest(point: tuple[float, float, float] | list[float], centers: list) -> int:
    best = 0
    best_d: float | None = None
    for j, center in enumerate(centers):
        d = _dist2(point, center)
        if best_d is None or d < best_d - 1e-9:
            best = j
            best_d = d
        elif abs(d - best_d) <= 1e-9 and j < best:
            best = j
            best_d = d
    return best


def _dist2(a, b) -> float:
    return (a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2 + (a[2] - b[2]) ** 2


def _dist(a, b) -> float:
    return _dist2(a, b) ** 0.5


def _shift_mode(points, start, bandwidth: float, max_iter: int) -> tuple[float, float, float]:
    limit = bandwidth * bandwidth
    cur = (float(start[0]), float(start[1]), float(start[2]))
    for _ in range(max_iter):
        sw = 0.0
        sx = 0.0
        sy = 0.0
        sz = 0.0
        for _key, center, weight in points:
            if _dist2(center, cur) <= limit:
                sw += weight
                sx += center[0] * weight
                sy += center[1] * weight
                sz += center[2] * weight
        if sw <= 0.0:
            break
        nxt = (sx / sw, sy / sw, sz / sw)
        if _dist2(nxt, cur) < 0.25:
            return nxt
        cur = nxt
    return cur


def _scan_clusters(
    width: int,
    height: int,
    pixels: list[tuple[int, int, int]],
    labels: list[int],
) -> list[dict]:
    buckets: dict[int, dict] = {}
    for i, lab in enumerate(labels):
        x = i % width
        y = i // width
        px = pixels[i]
        bucket = buckets.get(lab)
        if bucket is None:
            buckets[lab] = {
                "label": lab,
                "count": 1,
                "sum": [px[0], px[1], px[2]],
                "bbox": [x, y, x, y],
            }
            continue
        bucket["count"] += 1
        bucket["sum"][0] += px[0]
        bucket["sum"][1] += px[1]
        bucket["sum"][2] += px[2]
        bb = bucket["bbox"]
        if x < bb[0]:
            bb[0] = x
        if y < bb[1]:
            bb[1] = y
        if x > bb[2]:
            bb[2] = x
        if y > bb[3]:
            bb[3] = y
    clusters = []
    for lab in sorted(buckets):
        bucket = buckets[lab]
        count = bucket["count"]
        clusters.append(
            {
                "label": lab,
                "count": count,
                "centroid": [
                    round(bucket["sum"][0] / count, 4),
                    round(bucket["sum"][1] / count, 4),
                    round(bucket["sum"][2] / count, 4),
                ],
                "bbox": list(bucket["bbox"]),
            }
        )
    return clusters


def _empty_seg(algorithm: str, width: int, height: int, pixels, **extra) -> dict:
    body = {
        "algorithm": algorithm,
        "labels": [0] * (width * height),
        "clusters": _scan_clusters(width, height, pixels, [0] * (width * height)) if pixels else [],
    }
    body.update(extra)
    return body


def _public_segmentation(seg: dict) -> dict:
    body = {"algorithm": seg["algorithm"], "clusters": seg["clusters"]}
    if "k" in seg:
        body["k"] = seg["k"]
        body["seed"] = seg["seed"]
    if "bandwidth" in seg:
        body["bandwidth"] = seg["bandwidth"]
    if "bin" in seg:
        body["bin"] = seg["bin"]
    return body


def _bbox_area(bbox: list[int]) -> int:
    return (bbox[2] - bbox[0] + 1) * (bbox[3] - bbox[1] + 1)


def _fill(cluster: dict) -> float:
    area = _bbox_area(cluster["bbox"])
    if area <= 0:
        return 0.0
    return cluster["count"] / area


def _is_yellow(rgb: list[float]) -> bool:
    r, g, b = rgb
    return r >= 180.0 and g >= 160.0 and b <= 90.0 and r + 15.0 >= g


def _is_green(rgb: list[float]) -> bool:
    r, g, b = rgb
    return g >= 140.0 and r <= 120.0 and b <= 140.0 and g >= r + 40.0 and g >= b + 20.0


def _is_debug_block(rgb: list[float]) -> bool:
    r, g, b = rgb
    magenta = r >= 200.0 and b >= 200.0 and g <= 50.0
    cyan = g >= 200.0 and b >= 200.0 and r <= 50.0
    red = r >= 220.0 and g <= 30.0 and b <= 30.0
    blue = b >= 220.0 and r <= 30.0 and g <= 30.0
    return magenta or cyan or red or blue


def _touches_edge(bbox: list[int], width: int, height: int) -> bool:
    return bbox[0] == 0 or bbox[1] == 0 or bbox[2] == width - 1 or bbox[3] == height - 1


def _lod_chrome_hit(clusters: list[dict], width: int, height: int) -> bool:
    area = width * height
    if area <= 0:
        return False
    yellow = 0
    green = 0
    for cluster in clusters:
        frac = cluster["count"] / area
        if frac < LOD_EACH_FRAC_MIN or _fill(cluster) < LOD_FILL_MIN:
            continue
        if _is_yellow(cluster["centroid"]):
            yellow += cluster["count"]
        elif _is_green(cluster["centroid"]):
            green += cluster["count"]
    if yellow == 0 or green == 0:
        return False
    return (yellow + green) / area >= LOD_COMBINED_FRAC_MIN


def _overflow_hit(clusters: list[dict], width: int, height: int) -> bool:
    area = width * height
    if area <= 0:
        return False
    for cluster in clusters:
        frac = cluster["count"] / area
        if frac < DEBUG_AREA_FRAC_MIN or _fill(cluster) < DEBUG_FILL_MIN:
            continue
        if _is_debug_block(cluster["centroid"]) and _touches_edge(cluster["bbox"], width, height):
            return True
    return False
