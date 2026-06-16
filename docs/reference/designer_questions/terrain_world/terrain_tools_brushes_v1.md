# Designer Q — terrain **tools** (brushes + extended tooling) `v1`

---

## Locked intent

- Scope is **terrain tools**, not only brushes: brushes are the first slice; **same undo/command model** should cover **most tool actions** (paint, param edits, layer ops) as they are added.
- **Small initial brush/tool set** — raise/lower/flatten, biome paint, smooth (📎 exact list for v0).
- **Designed for expansion** — new tools register into a shared **command stack** / history.
- **Undo** — **required** for essentially all mutating tool actions; depth 📎 tunable (default stack size mid-range; can expose in settings later).

---

## Repo touchpoints

- World gen tools: `src/terrain/generation/world_generation_plugin.rs`, `src/gui/editor/world_gen_ui.rs`
- Overrides storage: ⏳ see `chunks_streaming_v1.md` + legacy B6.a

---

## Answers & implementation cues (append-only)

**Round:** 2026-04-27 · **For:** LLM + engineers

| Topic | Decision |
|:---|:---|
| Naming | Treat **“terrain tools”** as product name; “brushes” = subset. |
| Undo | **Command pattern** (or equivalent) per user action; merge micro-strokes if needed for memory. |
| Future tools | Noise re-roll, hydrology **preview** toggles, mask import — all go through same history interface. |

### 📎 Sub-questions

1. Undo **coalescing** — batch per stroke vs per tick?
2. **Redo** stack parity with undo?
