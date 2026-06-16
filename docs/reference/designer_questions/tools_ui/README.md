# Designer Q — tools, debug, performance UI

**Paired matrixes:** `prompts/matrix/engine_bevy/` (API constraints), `prompts/matrix/strategic_platforms/` (inspectors for platforms), `prompts/guides/ui_boundary_guide_v1.md` (Bevy vs egui rule)

> **Prompt use:** `ui_boundary_guide_v1.md` before mixing Bevy UI vs egui · cite plugin/panel paths · devtools must stay gated — `ASK:` if host unclear.

| File / folder | Role |
|:---|:---|
| **`spec/README.md`** | **Start here** — index `00`–`08` (plugins, split, cross-domain, metrics, asset studio, dual-chain audit, **world gen + terrain registries desktop**) |
| `spec/00_scope.md` … `spec/05_*.md` | Structured stubs |
| `debug_perf_ui_split_v1.md` | egui vs Bevy HUD; perf metrics; cross-domain tool rows |
| `tooling_cross_domain_v1.md` | Shared patterns: gen, build, vehicles, platforms |
| **`implementation_questions_v1.md`** | **Engineering checklist** |
