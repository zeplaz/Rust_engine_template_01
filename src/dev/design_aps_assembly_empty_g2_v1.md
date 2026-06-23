# APS assembly empty-state copy `v1` — tier-aware G2+ tail

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-ASSEMBLY-EMPTY-G2-001** |
| **Program** | APS-PRESENCE-CORRECTION-001 |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Parent** | [`design_aps_default_presence_audit_v1.md`](design_aps_default_presence_audit_v1.md) §1.3 |
| **Implements** | `AssemblyPanel` empty label in [`assembly_panel.py`](../../tools/mcp/art_pipeline_suite/assembly_panel.py) |
| **Handoff** | `@coder-mcp` — string + pytest match only |
| **Verdict** | **PASS** |

```text
DES-APS-ASSEMBLY-EMPTY-G2-001 Q✓
Empty assembly copy gains G2+ tail — no onboarding card change
```

---

## 0. Scope

**Copy-only** — one label on the assembly footprint grid when no building is loaded.

**Out of scope:** onboarding cards · kit hint strip · Set health wording (separate guards).

---

## 1. Strings (canonical)

| Tier band | Empty label (single line) |
|:---|:---|
| **G0–G1** | `No assembly yet — Generate one to begin.` |
| **G2+** | `No assembly yet — Generate one to begin, then tune shape bias in the panels below.` |

**Tier source:** same as exposure matrix — `grammar_set_tier()` live tier, not fixture JSON.

---

## 2. Placement

| Surface | Widget | When shown |
|:---|:---|:---|
| Assembly tab | Footprint grid center label | `assembly_snapshot is None` OR zero placements after clear |

**Do not** duplicate in onboarding step 3 — first-run cards stay tier-agnostic per audit §1.3.

---

## 3. Anti-patterns

| Ban | Why |
|:---|:---|
| Mention DNA / iterate panel names @ G0 | Overwhelms pilot tier |
| Engineer tier ids in label (`G3`, `grammar_set_tier`) | Artist-facing copy only |
| Different strings per archetype count | Tier band only |

---

## 4. Acceptance (coder-mcp)

| # | Check |
|:---:|:---|
| E1 | @ G0 fixture: label equals G0–G1 row exactly |
| E2 | @ G3 live tier: label equals G2+ row exactly |
| E3 | pytest `test_assembly_empty_label_tier_aware` (or equivalent) asserts both strings |
| E4 | No change to onboarding card copy |

---

## 5. Exit predicate

| Field | Value |
|:---|:---|
| **Deliverable** | `src/dev/design_aps_assembly_empty_g2_v1.md` (this file) |
| **Coder witness** | pytest green + label visible in headless assembly panel snapshot (optional) |
| **Designer closure** | Spec PASS + registry row **SIGNED** |

---

## 6. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-02 |
