# PLAN-DESIGNER-MCP-ART-TOOLCHAIN-001 — Designer MCP + Blender headless exec `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-DESIGNER-MCP-ART-001** |
| **Design sources** | [`prompts/art_desgin_inbound.md`](../../prompts/art_desgin_inbound.md) · [`prompts/designer_questions/art_design.md`](../../prompts/designer_questions/art_design.md) · [`art_extend.md`](../../prompts/designer_questions/art_extend.md) |
| **Alignment** | [`plan_art_design_inbound_alignment_v1.md`](plan_art_design_inbound_alignment_v1.md) |
| **Engine contract** | [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md) · [`06_asset_content_studio_workflow_v1.md`](../../prompts/designer_questions/tools_ui/spec/06_asset_content_studio_workflow_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@planner` + `@designer` (UX) · implementation `@coder` |
| **Status** | **SIGNED — READY** |
| **Horizon** | **~10 weeks** (foundation → Blender → full MCP stack) |
| **Repo root** | `tools/mcp/` (new) |

**Principle (from art_design):** The **Designer agent** writes **structured specifications** and reviews outputs. **Tools** (MCP + CLI) create assets. No “describe the mesh in chat” as production path.

---

## 1. Target pipeline

```text
Designer Agent (@designer + MCP tools)
        ↓
AssetSpec JSON (Art Director — never meshes)
        ↓
Job queue (orchestrator CLI / MCP)
        ↓
┌───────────────┬────────────────┬─────────────────┐
│ Geometry      │ Material       │ Reference       │
│ Blender       │ Material Maker │ OSM / NE / docs │
│ headless      │ / CLI PBR      │ (metadata only) │
└───────────────┴────────────────┴─────────────────┘
        ↓
assets/staging/<job_id>/
        ↓
Validation MCP (gltf, scale, PBR, naming, LOD)
        ↓
Promotion → assets/models/modules/ + RON sidecar
        ↓
Asset Library index (_module_index.ron)
        ↓
Bevy load (BuildingDefinition / StylePack / RepresentationResult)
```

**Forbidden:** AI image → final albedo; chat-only asset creation without `AssetSpec` + witness.

---

## 2. MCP architecture in Cursor

### 2.1 Why multiple MCP servers (not one mega-server)

Matches [`art_design.md`](../../prompts/designer_questions/art_design.md): separate concerns, separate process boundaries, designers enable only what they need.

| MCP server | Purpose | Calls external binary? |
|:---|:---|:---:|
| **art-director** | Create/review `AssetSpec`; style packs; **no** mesh IO | No |
| **asset-library** | List/register modules; `StylePack`; promotion status | No (filesystem index only) |
| **geometry-blender** | Queue Blender headless jobs; return job id + paths | **Yes** — `blender` |
| **material-pbr** | Queue Material Maker / texture CLI jobs | **Yes** — optional MM |
| **validation** | Run validators on staging path; return issue list | **Yes** — `gltf-transform`, custom |
| **reference** | Fetch **real** reference metadata (OSM, URLs, citations) | Network read-only |

Optional later: **houdini**, **substance** — same adapter pattern as Blender (§5).

### 2.2 Cursor wiring (project + user)

| Step | Action |
|:---|:---|
| 1 | Copy [`tools/mcp/cursor-mcp.example.json`](../../tools/mcp/cursor-mcp.example.json) → **user** `~/.cursor/mcp.json` **or** project `.cursor/mcp.json` (Cursor version dependent — use Settings → MCP if JSON merge is manual) |
| 2 | Set env vars in MCP config: `RUST_ENGINE_REPO`, `BLENDER_EXE`, `PYTHON` |
| 3 | `pip install -e tools/mcp/python` (editable package `rust_engine_mcp`) |
| 4 | Restart Cursor; confirm servers in MCP panel |
| 5 | Designer chat: `@designer` + enable **geometry-blender**, **validation**, **art-director** |

**Security:** All tool writes go under `assets/staging/` or `debug_runs/art_pipeline/` only. Promotion requires explicit `promote_asset` tool or CLI with `--confirm`.

### 2.3 Orchestrator option (recommended v1)

Single **rust-engine-art** MCP entry that **routes** to Python modules internally (one process, simpler Cursor config). Split into 6 servers in **Phase 4** when stable.

```
tools/mcp/python/rust_engine_mcp/
  server.py              # FastMCP or mcp SDK entry
  routers/
    art_director.py
    geometry.py
    material.py
    validation.py
    library.py
    reference.py
  adapters/
    blender_headless.py
    material_maker.py
    gltf_validator.py
```

---

## 3. Blender headless (Geometry backend)

### 3.1 Requirements

| Item | Value |
|:---|:---|
| **Blender** | 4.2+ LTS recommended (pin in `tools/mcp/blender/VERSION`) |
| **Mode** | `blender.exe --background --python script.py -- --job <path.json>` |
| **OS** | Windows primary (designer machine); Linux CI optional |
| **Output** | `.glb` (Draco optional Phase 2), export Y-up or Z-up per engine contract |

### 3.2 Job file contract (`tools/mcp/schemas/geometry_job_v1.json`)

```json
{
  "schema_version": 1,
  "job_id": "wall_brick_1u_001",
  "spec_ref": "assets/staging/specs/wall_brick_1u.json",
  "operation": "module_wall",
  "params": {
    "width_m": 4.0,
    "height_m": 3.0,
    "depth_m": 0.3,
    "material_profile": "brick_red_01"
  },
  "output": {
    "glb": "assets/staging/wall_brick_1u_001/model.glb",
    "thumbnail": "assets/staging/wall_brick_1u_001/preview.png"
  }
}
```

### 3.3 Blender scripts (repo-owned)

| Script | Role |
|:---|:---|
| `tools/mcp/blender/scripts/run_job.py` | argparse → dispatch by `operation` |
| `tools/mcp/blender/scripts/ops/module_wall.py` | Parametric wall (Geometry Nodes or mesh primitives v1) |
| `tools/mcp/blender/scripts/ops/module_roof.py` | Roof modules |
| `tools/mcp/blender/scripts/ops/export_glb.py` | glTF 2.0 export settings (game-ready) |
| `tools/mcp/blender/templates/` | Saved `.blend` node groups designers can extend |

**v1:** Python mesh primitives + modifiers (fast to ship). **v2:** Geometry Nodes templates per module category.

### 3.4 Launcher

| File | Use |
|:---|:---|
| `tools/mcp/blender/headless_run.ps1` | Windows: resolve `BLENDER_EXE`, run job, capture log → `debug_runs/art_pipeline/<job_id>.log` |
| `tools/mcp/blender/headless_run.sh` | Linux CI |

### 3.5 MCP tools (geometry)

| Tool | Args | Returns |
|:---|:---|:---|
| `geometry_submit_job` | `GeometryJob` JSON | `{ job_id, status: "queued" }` |
| `geometry_poll_job` | `job_id` | `{ status, log_tail, outputs[] }` |
| `geometry_list_templates` | `category?` | available `operation` ids |

---

## 4. Asset specification (Art Director — structured, token-cheap)

**Path:** `tools/mcp/schemas/asset_spec_v1.json` · examples in `assets/staging/specs/`

```json
{
  "schema_version": 1,
  "asset_id": "wall_brick_1u",
  "archetype": "module_wall",
  "style_pack": "style_industrial_west",
  "module": {
    "grid_units": [1, 1],
    "snap": "floor_edge",
    "pivot": "bottom_center"
  },
  "dimensions_m": { "w": 4, "h": 3, "d": 0.3 },
  "material_profile": "brick_red_01",
  "references": ["ref://osm/industrial_brick_warehouse"]
}
```

**MCP tools:** `spec_create`, `spec_validate`, `spec_diff` — **never** write binary.

**Engine mapping:** generate/update RON beside promoted glb:

`assets/configs/buildings/modules/wall_brick_1u.ron` — fields from [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md).

---

## 5. Adapter pattern (other open-source tools)

Every backend implements:

```python
class ToolAdapter(Protocol):
    def capabilities(self) -> list[str]: ...
    def submit(self, job: dict) -> str: ...  # job_id
    def poll(self, job_id: str) -> JobStatus: ...
```

| Adapter | Binary | Phase | Role |
|:---|:---|:---:|:---|
| `BlenderAdapter` | `blender` | **1** | Geometry |
| `GltfTransformAdapter` | `gltf-transform` | **1** | Validate, resize, compress |
| `MaterialMakerAdapter` | Material Maker CLI | **2** | PBR texture sets |
| `ImageMagickAdapter` | `magick` | **2** | Thumbnails, channel merge |
| `MeshoptAdapter` | `gltfpack` | **3** | LOD / meshopt |
| `CustomRustAdapter` | `cargo run -p art_validator` | **3** | Engine-specific rules (grid, pivot) |
| `HoudiniAdapter` | `hython` | **4+** | Optional high-end |
| `SubstanceAdapter` | `sbsrender` | **4+** | Optional |

**Build our own:** `tools/art_validator/` Rust crate — polygon budget, pivot, 1m grid snap — invoked by validation MCP (no Blender required).

---

## 6. Validation MCP (production gate)

Checks from art_design:

| Check | Tool |
|:---|:---|
| glTF valid | `gltf-transform validate` |
| Scale / units | Rust `art_validator` |
| PBR textures present | manifest vs spec |
| Naming convention | `module_id` regex |
| LOD0 poly budget | per category table |
| Collision mesh | optional second glb |
| Grid alignment | bbox snap 1m |

**Witness:** `debug_runs/art_pipeline_validation_live.json` — `{ asset_id, valid, issues[] }`.

**MCP:** `validate_asset(path)` → structured result; designer agent fixes spec and re-runs.

---

## 7. Asset Library MCP

**Index:** `assets/configs/buildings/_module_index.ron` (already planned in procedural kit).

| Tool | Action |
|:---|:---|
| `library_search` | by `style_tags`, category |
| `library_register` | after promotion — add id + paths |
| `library_style_pack_get` | return module sets for Victorian / Industrial / … |

Aligns with **assemble, don’t prompt** — `StylePack` references module ids, not “create apartment.”

---

## 8. Reference MCP (real sources only)

| Source | v1 | Output |
|:---|:---:|:---|
| OpenStreetMap Overpass | metadata + tags | JSON citations, no tiles |
| Natural Earth | layer names + URLs | bibliography for designer |
| Local docs / manuals | file paths in repo | excerpt + page |

**No AI imagery.** `reference_gather(spec)` → `reference_set[]` with URLs and labels for designer review in Asset Content Studio.

---

## 9. Integration with existing repo

| Existing | Integration |
|:---|:---|
| `src/utils/asset_tools/` (PyQt) | “Open staging folder”, import promoted glb, edit sidecar JSON |
| `assets/configs/buildings/` | Promotion target + StylePack RON |
| `design_procedural_module_kit_v1.md` | 50 module targets = first Blender template pack |
| `construction_product_roadmap` Phase 4 | Placeholder kit fed by this pipeline |
| Stage 5 `RepresentationResult` | glb path + material tags in catalog |
| `debug_runs/` | All pipeline witnesses per agent debug envelope |

---

## 10. Implementation phases (coder workboard)

### Phase 0 — Repo scaffold + Cursor MCP (week 1)

| ID | Deliverable | Owner |
|:---|:---|:---|
| **ART-MCP-000** | `tools/mcp/README.md`, schemas, example mcp.json | A |
| **ART-MCP-001** | `rust_engine_mcp` Python package; `mcp` dev deps in `tools/mcp/requirements.txt` | A |
| **ART-MCP-002** | Single orchestrator MCP: `ping`, `spec_validate` | A |
| **ART-MCP-003** | Designer doc: enable MCP in Cursor (Windows) | designer |

### Phase 1 — Blender headless + validation CLI (weeks 2–3)

| ID | Deliverable | Owner |
|:---|:---|:---|
| **ART-GEO-001** | `headless_run.ps1` + `run_job.py` + `module_wall` op | B |
| **ART-GEO-002** | MCP `geometry_submit_job` / `geometry_poll_job` | B |
| **ART-VAL-001** | `gltf-transform` + staging validator script | A |
| **ART-VAL-002** | MCP `validate_asset` | A |
| **ART-GEO-003** | 3 template ops: wall, roof, door (greybox) | B |

### Phase 2 — Spec + library + promotion (weeks 4–5)

| ID | Deliverable | Owner |
|:---|:---|:---|
| **ART-SPEC-001** | `AssetSpec` JSON schema + examples (10 modules) | A |
| **ART-LIB-001** | `promote_staging_asset.ps1` → modules + RON sidecar | B |
| **ART-LIB-002** | MCP `library_register`, `library_search` | B |
| **ART-DIR-001** | MCP `spec_create` from StylePack templates | A |

### Phase 3 — Material pipeline (weeks 6–7)

| ID | Deliverable | Owner |
|:---|:---|:---|
| **ART-MAT-001** | Material Maker adapter (or ImageMagick fallback PBR pack) | B |
| **ART-MAT-002** | MCP `material_generate_set` | B |
| **ART-MAT-003** | Link material profiles to `assets/configs/buildings/materials/` | A |

### Phase 4 — Reference + split MCP servers (weeks 8–9)

| ID | Deliverable | Owner |
|:---|:---|:---|
| **ART-REF-001** | OSM Overpass reference adapter (read-only) | B |
| **ART-MCP-010** | Split orchestrator → 6 Cursor MCP entries (optional) | A |

### Phase 5 — Designer production kit (week 10)

| ID | Deliverable | Owner |
|:---|:---|:---|
| **ART-KIT-001** | 10+10+10+10+10 greybox modules per procedural kit | designer + GEO |
| **ART-KIT-002** | Witness batch `art_pipeline_batch_001.json` all valid | designer |

---

## 11. PR / ownership split

| Coder | Territory |
|:---|:---|
| **A** | Python MCP SDK, validation, art_validator Rust, schemas, promotion CLI |
| **B** | Blender scripts, geometry adapter, material adapters, reference fetch |

**Designer** owns: Blender `.blend` templates, StylePack art direction, module visual QA — not MCP server code.

---

## 12. Environment variables

| Var | Example | Required |
|:---|:---|:---:|
| `RUST_ENGINE_REPO` | `C:\dev\github\Rust_engine_template_01` | yes |
| `BLENDER_EXE` | `C:\Program Files\Blender Foundation\Blender 4.2\blender.exe` | geometry |
| `MATERIAL_MAKER_EXE` | path to Material Maker CLI | optional |
| `GLTF_TRANSFORM` | `npx gltf-transform` or global bin | validation |

Document in [`tools/mcp/README.md`](../../tools/mcp/README.md) and `runtime_env_policy_registry` row **ART-*** (designer ops).

---

## 13. Regression

```powershell
cd tools/mcp/python
pip install -r ../requirements.txt
python -m pytest tests/

# Blender smoke (machine with BLENDER_EXE):
.\tools\mcp\blender\headless_run.ps1 -Job tools/mcp/schemas/examples/wall_job.example.json
```

---

## 14. Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Full MCP + Blender headless program from art_design.md |
