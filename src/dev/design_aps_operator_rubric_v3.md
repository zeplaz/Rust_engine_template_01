# APS operator building-quality rubric `v3` — golden eyes

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-OPERATOR-RUBRIC-003** |
| **Queue** | **APS-GOLDEN-RUBRIC-OPS-001** · `tools/orchestrator/queues/mcp_aps_tooling_finish_queue.json` |
| **Program** | PLAN-MCP-APS-TOOLING-FINISH-001 · BQ-Q3 / APSR-Q3 |
| **Date** | 2026-09-24 |
| **Owner** | `@designer` (charter) · `@operator` (eyes) |
| **Skill** | `aps-design-ux` |
| **Supersedes** | [`design_aps_operator_rubric_v2.md`](design_aps_operator_rubric_v2.md) for **BQ-Q2 / BQ-Q3 building quality** only — v2 MIN window walk remains UI chrome |
| **Needs display** | true — operator session with Assembly preview + Golden seed panel |
| **Verdict** | **READY** (charter shipped · operator pending · `operator_pass=false`) |

```text
DES-APS-OPERATOR-RUBRIC-003 Q✓ charter
Criterion — "reads as a real building"
Sheet — debug_runs/aps_golden_seed_rubric_rows.json
Scaffold — debug_runs/aps_golden_rubric_sheet_scaffold_live.json
⛔ pytest / machine approve never closes BQ-Q3
```

---

## 0. Intent (why v3)

v2 is a **pixel walk of APS chrome** (welcome, tabs, disabled toasts).  
v3 is the **building-quality gate**: human judgment that a golden seed **reads as a real building**, not hash-green or pytest theater.

| Authority | Role |
|:---|:---|
| Machine scaffold | Expands 12 golden seeds → `pending_operator` rows · `operator_pass=false` forever from scaffold |
| This charter | How to look · what fails · what to write on each sheet field |
| `@operator` session | Writes `approve` / `reject` with human `note` · `recorded_by=operator` |
| Pytest | May exercise write path with `note=pytest` — **demoted / never counts** |

---

## 1. Sheet field contract (wire to scaffold)

Authority files:

- Rows: `debug_runs/aps_golden_seed_rubric_rows.json`
- Scaffold witness: `debug_runs/aps_golden_rubric_sheet_scaffold_live.json`
- CLI: `aps-golden-rubric-scaffold` (`rust_engine_mcp.golden_seed_review`)

### 1.1 Sheet envelope

| Field | Operator rule |
|:---|:---|
| `version` | `2` |
| `task_id` | `APS-GOLDEN-RUBRIC-OPS-001` |
| `criterion` | Exact string **`reads as a real building`** |
| `operator_pass` | Stay **`false`** until §5 exit predicate — designer charter does **not** flip this |
| `scaffold_only` | `true` while any row is pending or theater |
| `rows[]` | One row per golden `seed_key` (≥12) |

### 1.2 Per-row fields

| Field | Write when | Allowed values / meaning |
|:---|:---|:---|
| `seed_key` | scaffold | `{archetype_id}:{district_style}:s{seed}` — do not invent |
| `archetype_id` | scaffold | From BQ-Q3 golden set |
| `district_style` | scaffold | From golden set |
| `seed` | scaffold | Integer seed |
| `expected_hash` | scaffold | Assembly hash under review — reject if preview hash≠expected without note |
| `criterion` | always | **`reads as a real building`** |
| `verdict` | operator | `pending_operator` → `approve` \| `reject` only |
| `screen_pass` | operator | `true` **only** with usable preview (Assembly load **or** Q2 PNG) **and** `verdict=approve` |
| `note` | operator | Plain-language why · **never** `pytest` · cite fail codes G1–G8 or S1–S4 |
| `recorded_at` | auto | ISO UTC |
| `rubric_ref` | scaffold/refresh | **`src/dev/design_aps_operator_rubric_v3.md#BQ-Q3`** |
| `recorded_by` | operator | **`operator`** — never `pytest` / `coder-mcp-machine` / `machine` for pass credit |

### 1.3 Theater rejection (instant non-count)

A row **does not** count toward `operator_pass` when any of:

- `note` equals / contains pytest as approve theater
- `recorded_by` ∈ {`pytest`, `coder-mcp-machine`, `machine`}
- `verdict` missing or still `pending_operator`

Scaffold CLI demotes theater → `pending_operator`. Re-run after any accidental pytest approve.

---

## 2. Session setup (operator)

| Setting | Value |
|:---|:---|
| Launch | `python -m art_pipeline_suite.run` · Buildings domain |
| Window | ≥1280×720 (MIN ok if Golden seed + QC strip readable) |
| Panel | Assembly → **Golden seed review (BQ-Q3)** |
| QC strip | **Assembly QC (BQ-A2 / Q2)** visible under Assembly |
| Evidence | Prefer **Load seed** → live Assembly preview; optional Q2 PNG under `debug_runs/bq_q2_screens/` |
| Sheet refresh | `aps-golden-rubric-scaffold` if rows missing / theater present — then stop; eyes only |

**Do not** close BQ-Q3 from cargo/pytest green alone.

---

## 3. Criterion — “reads as a real building”

Judge the **composed assembly**, not chrome polish.

### 3.1 Approve checklist (all must hold)

| # | Look for | Pass if |
|:---:|:---|:---|
| A1 | **Massing** | Footprint reads as one building (or intentional cluster) — not scattered boxes |
| A2 | **Facade order** | Openings / walls / roofs feel structured; street face readable |
| A3 | **Style coherence** | Materials/modules match district style (no obvious cross-pack jumble) |
| A4 | **Silhouette** | Continuity at mid zoom — no impossible floating slabs / hollow shell |
| A5 | **Grounding** | Sits on ground plane; no float/sink that breaks scale trust |
| A6 | **Slots honest** | No silent missing door/window that should exist for archetype |
| A7 | **QC strip** | No red smoke / ship-block label (§4) · A2 score not an instant fail alone if eyes still pass — note score |
| A8 | **Preview trust** | Loaded preview matches `seed_key`; hash drift noted if reject |

### 3.2 Reject codes (any one ⇒ `verdict=reject`)

| Code | Fail | Note hint |
|:---|:---|:---|
| **G1** | Incoherent jumble / unreadable massing | `G1 massing jumble` |
| **G2** | Cross-style kit collision (e.g. victorian brick + industrial roof as accident) | `G2 cross-style` |
| **G3** | Broken silhouette / floating / sunk | `G3 silhouette` |
| **G4** | Missing critical slot (door/window/roof) left as hole or grey void | `G4 missing slot` |
| **G5** | Scale wrong (toy / skyscraper sill) | `G5 scale` |
| **G6** | Smoke / greybox module still in player-facing compose | `G6 smoke-tier` + §4 label |
| **G7** | Hash/preview mismatch without intentional regen | `G7 hash drift` |
| **G8** | Cannot judge — blank preview, no PNG, load failed | keep `pending_operator` or `reject` with `G8 no eyes` · `screen_pass=false` |

---

## 4. APS QC strip — smoke / tier labels (copy charter)

Strip widget: `AssemblyQcStrip` · text from BQ-A2 + Q2 helpers.  
**Designer owns labels**; `@coder-mcp` wires strings if strip still omits tier.

### 4.1 Required human labels (when tier hits present)

| Condition | Strip label (exact intent) | Severity |
|:---|:---|:---|
| Any placement `development_tier=smoke` or batch `kit_greybox*` / `kit_smoke*` | **`Smoke kit — not ship`** | Block approve |
| Any placement `development_tier=lod0` on CORE slot after promote waves | **`LOD0 stand-in — production pending`** | Warn; reject if reads greybox (G6) |
| `missing_slot_count > 0` | **`Missing slots: N`** (already numeric — keep plain) | Soft fail → check G4 |
| `adjacency_violation_count > 0` | **`Adjacency: N`** | Soft fail |
| Style purity &lt; gate | **`Style purity low`** + % | Soft fail → check G2 |
| Q2 PNG present, criterion open | **`reads as a real building: pending_operator`** | Informational |
| Q2 no PNG | **`No screenshot — cannot pass`** | Block `screen_pass` |

### 4.2 Interaction rules

```text
○idle QC strip ─load/generate▶ ◐refresh
  ═[smoke hit]▶ ⊘blocked: "Smoke kit — not ship"  ⛔ Approve golden / Approve snapshot
  ═[lod0 CORE]▶ ◐warn: "LOD0 stand-in — production pending"
  ═[eyes approve + no smoke]▶ ★ screen_pass may become true
```

- Hue alone is insufficient — label text + block/warn state required (a11y).
- Smoke hit ⇒ operator **must** `reject` with **G6** (or leave pending if load failed — G8).
- Closing index smoke count to 0 does **not** auto-approve rows — eyes still required.

### 4.3 Handoff for strip wire (if missing)

| ID | Owner | Work |
|:---|:---|:---|
| **APS-QC-SMOKE-LABEL-001** | `@coder-mcp` | **DONE** — `format_qc_strip_text` emits §4.1 labels; smoke blocks approve · `debug_runs/aps_qc_smoke_label_001_live.json` |

---

## 5. Operator walk (≤25 min · 12 seeds)

| # | Step | Sheet write |
|:---:|:---|:---|
| O1 | Open Golden seed list — confirm 12 human labels (archetype · district · seed) | — |
| O2 | Confirm scaffold `pending_operator_count ≥ 12` · `operator_pass=false` | — |
| O3 | For each seed: **Load seed** → watch QC strip · inspect massing | — |
| O4 | If A1–A8 pass and no G*/S* | `verdict=approve` · `screen_pass=true` · `note` ≤1 line · `recorded_by=operator` |
| O5 | If any G1–G7 | `verdict=reject` · `screen_pass=false` · `note` starts with code |
| O6 | If G8 | do **not** approve · fix preview / re-run Q2 · leave pending |
| O7 | After all 12 human verdicts | `@operator` closes **BQ-Q3-OPS-APPROVE-001** with witness `debug_runs/bq_q3_operator_approve_live.json` |

### Style-pack coverage (eyes)

Golden set must visually cover industrial + manufacturing / warehouse clusters present in seeds — do not approve “all warehouse look identical mush” without checking district labels.

---

## 6. Exit predicates

### 6.1 Designer (this charter) — **SHIPPED when**

| Predicate | Status |
|:---|:---|
| `src/dev/design_aps_operator_rubric_v3.md` exists | ✓ this file |
| Scaffold `rubric_ref` may resolve to v3 (`designer_rubric_v3_exists=true` after refresh) | after CLI |
| `operator_pass` remains **false** | mandatory |
| Queue **APS-GOLDEN-RUBRIC-OPS-001** designer portion → done | this session |

### 6.2 Operator / BQ-Q3 — **NOT claimed by designer**

| Predicate | Closes |
|:---|:---|
| ≥12 rows with `is_operator_verdict` | `operator_pass=true` in sheet |
| Zero pytest theater rows | required |
| Zero `pending_operator` | required |
| Witness `debug_runs/bq_q3_operator_approve_live.json` | **BQ-Q3-OPS-APPROVE-001** |

---

## 7. Anti-patterns (instant process fail)

| # | Fail |
|:---:|:---|
| F1 | Marking BQ-Q3 / `operator_pass` from scaffold or pytest |
| F2 | Approving without loading preview or viewing Q2 PNG |
| F3 | Approving through **Smoke kit — not ship** |
| F4 | `recorded_by=machine` with `verdict=approve` left in sheet |
| F5 | Treating A2 score green as pixel pass |
| F6 | Editing `expected_hash` to force match instead of reject/regen |

---

## 8. Diagnostics / witnesses

| Artifact | Role |
|:---|:---|
| `debug_runs/aps_golden_rubric_sheet_scaffold_live.json` | Machine-green sheet ready · `operator_pass=false` |
| `debug_runs/aps_golden_seed_rubric_rows.json` | Live verdict sheet |
| `debug_runs/bq_q3_golden_001_live.json` | Golden seed set (hashes) |
| `debug_runs/bq_q2_screen_001_live.json` | Screenshot evidence lane |
| `debug_runs/building_quality_live.json` | A2 scores feeding QC strip |
| `debug_runs/bq_smoke_tier_audit_live.json` | Tier inventory (smoke should be 0 post archive) |

---

## 9. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **READY** — charter + sheet field map + QC smoke labels | 2026-09-24 |
| `@operator` | **PENDING** — BQ-Q3-OPS-APPROVE-001 display session | — |
| `@coder-mcp` | **DONE** — APS-QC-SMOKE-LABEL-001 strip labels wired · `aps_qc_smoke_label_001_live.json` | 2026-09-24 |

**Unblocks:** operator eyes on **BQ-Q3-OPS-APPROVE-001**  
**Does not unblock:** ship / `proceed_ship` / art-ship-green — eyes first.

---

## Anchor fragments

```text
#BQ-Q3  — golden-seed sheet walk (§1–§6)
#BQ-Q2  — screenshot evidence · screen_pass rules (§1.2 · §4)
#QC     — Assembly QC strip smoke/lod0 labels (§4)
```
