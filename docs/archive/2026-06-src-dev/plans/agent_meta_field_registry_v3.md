```text
REGISTRY◈v3Ω  ◉Bind↑↑  ◉Drift↓↓  REPO=Rust_engine_template_01
PARENT→ agent_meta_brief_v3.md  LANG→ agent_lang_v1.md  ENTRY→ llm_agent_brief.md
FOLD→ agent_meta_diagrams_v3_fold.md  (multi-line topology — ⛔ one-line dumps)
GRAM→ agent_meta_grammar_v3_lattice.md  (STATE·FLOW·REVIEW clusters)
```

| Field | Value |
|:---|:---|
| **ID** | **AGENT-META-REGISTRY-003** |
| **Status** | **ACTIVE** 🟢 |

---

## 1. Core dimensions `FIELD◈` — ΩMETA-LATTICE

```text
{Ct, Cx, Cm, Dp, Au, Rk, Q, Δ, U, T, E, H, S}
```

**Grammar:** [`agent_meta_grammar_v3_lattice.md`](agent_meta_grammar_v3_lattice.md)

| Sym | Name | Emoji | v1 alias | Measure 🔬 |
|:---:|:---|:---:|:---|:---|
| **Ct** | TokenCost | 💰📉 | burn | `token_savings_guide()` |
| **Cx** | Complexity | 🌀🕸⊗ | coupling | budget paste |
| **Cm** | Maintenance | 🌊♻🔥 | debt | cleanup-intelligence |
| **Dp** | Dependency | ⛓🔗⊗ | deps | matrix + `$ref:` |
| **Au** | Authority | 🏛⊚⬇ | AUTH | bevy-sim `07` |
| **Rk** | Risk | 🕳⚠☋ | constraint | debug-intelligence |
| **Q** | Quality | 🎯📈🌟 | gain | compress=4 |
| **Δ** | Change | ⚖ | BLANG:RUN | diff · witness |
| **U** | Uncertainty | ⌁🧠? | — | review tags |
| **T** | Time | ⏱⟳ | — | drift |
| **E** | Evidence | 🧪🔬📜 | 🟢 | WIT · CARGO |
| **H** | HumanImpact | 💬📎👁 | 💬 | ask · UX |
| **S** | Scale | 🌐S+ | — | OPS · multiview |

```text
OBJECTIVE◈ Σ(Q+Au+E+Veracity+H) − Σ(Ct+Cx+Cm+Rk+U×T+Entropy)
```

---

## 1b. STATE-GRAMMAR◈ · 1c. FLOW-GRAMMAR◈ · 1d. REVIEW-TAGS◈

Normative tables: [`agent_meta_grammar_v3_lattice.md`](agent_meta_grammar_v3_lattice.md)

| Cluster | Packet |
|:---|:---|
| `🟢✅⊚` | verified + authority valid |
| `🟡⏳⊗☊` | partial · dep · constraint |
| `🔴❌🏛` | authority violation |
| `⊚A═▶⊚B` | hard authority flow |
| `🧠?` `⊗!` `⚡!` | review tags |

---

## 2. STATUS lexicon — v3 ↔ v1 ↔ emoji

```text
✅🟢 verified    ⏳🟡 partial    ⚠🟡 unverified    📎💬 ask    ❌🔴⛔ forbidden
♻🧊 candidate    🔬 measure     🧪 tests         🔥 debt     🏛 authority
🕸 dependency    💰 cost        📈 gain          🚦⚡P0 gate
```

| v3 | v1 | Scene |
|:---:|:---:|:---|
| ✅ | 🟢 | witness green · validator pass |
| ⏳ | 🟡 | qualified · partial |
| ⚠ | 🟡+note | needs BLANG:CARGO/WIT |
| 📎 | 💬 | human gate · no invented Hz |
| ❌ | 🔴+⛔ | banned import · layer invert |
| 🔬 | BLANG:CARGO/BEVY | validation-first |
| 🧪 | BLANG:S5/PY | stage5 · pytest |
| 🚦 | ⚡P0 | `validate_p0_gate_plain` |

---

## 3. 🏛 AUTHORITY-MAP → repo layers

```text
◇Designer → 📘Prompt → 📊Matrix → 📐Spec → ⚙ImplQ → 💻Code → 🎮Runtime → 🔬Validate → 📜Witness
     ⛔ UI→ECS    ⛔ ECS→Serializable    ⛔ Runtime→Spec
     ⊚ SRC > Matrix > Spec > README > Prompt > Memory
```

| Layer ↓ | Repo ⊚ | Skill | @agent |
|:---:|:---|:---|:---|
| Designer | `prompts/designer_questions/**` | — | `@designer` |
| Matrix | `prompts/matrix/**` | — | `@planner` |
| Spec | `prompts/**/spec/` · `src/dev/*_spec_*.md` | — | `@planner` |
| ImplQ | `implementation_questions_v1.md` | — | `@coder` |
| Code | `src/` · `tools/mcp/` | bevy-sim · mcp-* | `@coder` `@coder-mcp` |
| Runtime | Bevy · `--test` | sim-steward | `@sim-steward` |
| Validate | validation-first | validation-first | all |
| Witness | `debug_runs/**` | debug-intelligence | `@sim-steward` |

---

## 4. TASK-ROUTER◈ → matrix paths

```text
Terrain🌍───▶ terrain_biome_migration_matrix_v1.md ⊗ terrain_world/README.md
Prod🏭──────▶ production_migration_matrix_v1.md ⊗ production_economy/README.md
Nav🧭───────▶ repo_boundary_matrix_v1.md ⊗ navigation/README.md
Faction⚔───▶ serialization/ + assets/ ⊗ factions/README.md
Strategic🎯─▶ strategic_platforms_matrix_v1.md ⊗ strategic_platforms/README.md
UI🎨───────▶ ui_boundary_guide_v1.md ⊗ tools_ui/README.md
Save💾──────▶ serialization_hybrid_migration_matrix_v1.md
Assets📦────▶ bevy_asset_config_migration_matrix_v1.md
Bevy⚙──────▶ bevy_0_18_migration_plan.md
OPS📜──────▶ OPS_WITNESS_SPINE.md
MCP-Art🖼──▶ MICRO_TOOLS_REGISTRY_v1.md
```

⛔ **Ct guard:** ONE domain cluster per slice · cross-domain = `🔗` + `⟨ID⟩`

---

## 5. STACK◈ → evidence chain

```text
📘Design → 📐Spec → ⚙Impl → 🔬Verify → 🧪Test → 📜Witness
   │         │        │         │         │         └─ debug_runs/*_live.json
   │         │        │         │         └─ cargo test / pytest
   │         │        │         └─ validate-report compress=4
   │         │        └─ src/ + $sym:
   │         └─ *_spec_*.md
   └─ matrix + designer README
```

Skip = ☋ **LayerViolation**

---

## 6. ROLE-GRAPH → agents

```text
@orchestrator⚑ → @planner🏛 → @coder⚙ → @sim-steward🧠 → 🔬validate → 📜witness
```

| Role | File | Emit |
|:---|:---|:---|
| Planner | `orchestrator.md` | ⚑ scope · EV/Cx |
| Architect | `planner.md` | 🏛 · 🕸 |
| Implementer | `coder.md` | Δ · $sym: |
| Reviewer | `sim-steward.md` | ☍ |
| Validator | validation-first | 🔬 JSON |
| Witness | debug-intelligence | 📜 path |

---

## 7. DECISION-FORMULA◈

```text
Utility = (Q × Confidence × Reuse) / (Ct + Cx + Cm + Dp + Rk)
EV/Cx ≥ 1.0 → ✅ APPROVE │ 0.5–1.0 → ⚠ REVISE │ <0.5 → 🧊 DEFER
```

---

## 8. 💰 TOKEN-ECONOMY → BLANG

```text
▮ ReadDoc      → BLANG:REF · $ref:§
▮▮ ReadSrc     → $sym:@path
▮▮▮ Trace      → grep · semantic
▮▮▮▮ Design    → matrix cite only
▮▮▮▮▮ CrossSys → orchestrator + conflict matrix
▮▮▮▮▮▮ Rewrite → ⛔ without REVISE gate
```

---

## 9. HEALTH-FIELD◈ paste (vertical — not one line)

```text
╔═ HEALTH◈ ═══════════════════════════════════════════════════════════════╗
║  Au     [█████░░░░░]  🏛  authority                                     ║
║  Cx     [██████░░░░]  🕸  complexity                                    ║
║  Cm     [████░░░░░░]  🔥  maintenance                                   ║
║  Dp     [███████░░░]  🔗  dependency                                    ║
║  Ct     [█████░░░░░]  💰  token cost                                     ║
║  Verify [███░░░░░░░]  🔬  evidence                                      ║
╚═════════════════════════════════════════════════════════════════════════╝
```

Atlas: `agent_meta_diagrams_v3_fold.md` §HEALTH · §STATUS-FOLD · §RESPONSE-CONTRACT

---

## 10. ☋ FAILURE-TREE → repair

```text
❌NoSource🧾 ──┐
❌WrongAuth🏛 ──┼──▶☋BadDecision──▶♻Rework──▶💰↑──▶📉Trust
❌DocDrift📄 ──┘
```

| ☋ | ♻ Repair |
|:---|:---|
| NoSource🧾 | `$ref:` · 📎 |
| WrongAuth🏛 | ui_boundary |
| AssumptionLeak | ASK: |
| DocDrift📄 | grep Applied |
| TokenWaste💰 | BLANG:WIT |
| LayerViolation | boundary matrix |
| MissingEvidence | BLANG:S5 |

---

## 11. KNOWLEDGE-FLOW ↔ MCP

```text
Source⊚ → BLANG:REF → ⚑Decision → ΔChange → BLANG:CARGO → BLANG:WIT → BLANG:MARK → OPS↺
```

| Stage | Tool |
|:---|:---|
| Extract | `agent_doc_touch(ref)` |
| Queue | `agent_queue_next` · `handoff_brief` |
| Validate | `validate_cargo_report(4)` |
| Witness | `witness_brief` |
| Telemetry | `agent_run_append` · `agent_marker_append` |

---

## 12. MICRO-GRAPH◈ repo

```text
Designer◇→APS★→Snapshot⊚→Worker⚙@coder-mcp→Atlas★→Runtime○
Telemetry↺ KPI↺ Supervisor↺ HANDOFF↺
MAT⊗VAR⊗LOD → master_chain_tensor_v1.json
Validator☍Worker → validate-report → witness
```

---

## Changelog

| Version | Date | Note |
|:---|:---|:---|
| v3.0.0 | 2026-06-07 | Initial registry |
| v3.1.0 | 2026-06-07 | Full emoji · ASCII topology · status constellation |
| v3.2.0 | 2026-06-07 | Fold atlas link · vertical HEALTH · RESPONSE template |