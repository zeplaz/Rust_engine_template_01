# mod.rs spectrum audit `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **MOD-RS-SPECTRUM-AUDIT-001** |
| **Date** | 2026-06-10 |
| **Scope** | All `src/**/mod.rs` (91 files) |
| **Owner** | `@sim-steward` / `@coder` |

---

## Executive summary

**Rust norm:** `mod.rs` should **declare submodules** and **re-export** — not hold hundreds of lines of logic. This repo mixes three patterns:

| Pattern | Verdict | Count (approx.) |
|:---|:---|:---:|
| Thin barrel (mods + `pub use` only) | **Good** | ~40 |
| Medium plugin root (~100–400 lines) | **Acceptable** | ~25 |
| God-module (500+ lines, many fns) | **Refactor** | **6** |

**No action on `src/dev/mod.rs`** — it is a re-export hub only (correct for witness/todo crates).

---

## P0 — split candidates (logic belongs in sibling modules)

| File | Lines | fn | Issue | Suggested split |
|:---|---:|---:|:---|:---|
| [`map_editor/mod.rs`](../gui/editor/map_editor/mod.rs) | **1604** | 40 | Entire map editor tool, roads, brush, undo, plugins in one file | `tool.rs`, `road.rs`, `brush.rs`, `undo.rs`, `plugin.rs`, `dev_io.rs` |
| [`world_preview/mod.rs`](../gui/editor/world_preview/mod.rs) | **683** | 29 | D02/D04/D07 layout + plugin + chrome lifecycle | `layout.rs`, `chrome.rs`, `plugin.rs` (exists partially — consolidate) |
| [`io/streaming/mod.rs`](../io/streaming/mod.rs) | **791** | 18 | Streaming hydrate/flush monolith | `hydrate.rs`, `flush.rs`, `plugin.rs` |
| [`minimap_compositor/mod.rs`](../render/minimap_compositor/mod.rs) | **614** | 18 | Compositor + extract in mod root | `extract.rs`, `compositor.rs`, `plugin.rs` |
| [`strategic/mod.rs`](../strategic/mod.rs) | **455** | 5 | Large plugin + systems inline | move systems to `systems.rs` |
| [`terrain/mobility/mod.rs`](../terrain/mobility/mod.rs) | **330** | 15 | Domain logic in mod | `policy.rs`, `plugin.rs` |

---

## P1 — monitor (acceptable short-term, split when next touched)

| File | Lines | Notes |
|:---|---:|:---|
| [`construction/mod.rs`](../construction/mod.rs) | 379 | Plugin + funnel — keep plugin thin |
| [`gui/style/mod.rs`](../gui/style/mod.rs) | 316 | Theme tokens + helpers |
| [`infrastructure/profiles/mod.rs`](../infrastructure/profiles/mod.rs) | 321 | Profile registry |
| [`gui/map_view/mod.rs`](../gui/map_view/mod.rs) | 168 | Submodules exist — ensure new code goes there |

---

## P2 — good barrels (do not refactor)

| File | Lines | Role |
|:---|---:|:---|
| [`dev/mod.rs`](../dev/mod.rs) | 154 | `pub mod` + `pub use` witnesses/todos only |
| [`render/mod.rs`](../render/mod.rs) | 350 | Re-exports render spine |
| [`gui/mod.rs`](../gui/mod.rs) | 262 | Plugin graph + re-exports |
| [`gui/hud/mod.rs`](../gui/hud/mod.rs) | 193 | Submodule declarations |
| [`dev/runtime_witness/mod.rs`](../dev/runtime_witness/mod.rs) | 106 | Shim re-exports |

---

## Rules for new code

1. **`mod.rs` max ~80 lines** of non-re-export code; otherwise add a sibling file.
2. **Plugins:** `impl Plugin` lives in `plugin.rs` or `{domain}_plugin.rs`, not at bottom of 1k-line `mod.rs`.
3. **Tests:** `#[cfg(test)] mod tests` at bottom of **feature file**, not in crate-root `mod.rs` unless tiny.
4. **Docs:** When splitting, leave a one-line comment in `mod.rs` pointing to the owner file.

---

## Verification (after P0 splits)

```powershell
cargo test -p proc_A_dine01 --lib map_editor world_preview streaming minimap
```

---

## Changelog

| Ver | Note |
|:---|:---|
| v1 | Automated line/fn scan + steward classification |
