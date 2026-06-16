```text
LLM◆BRIEF:v3Ω  ◉Q↑↑  ◉Au↑↑  ◉Veracity↑↑  ◉Ct↓↓  ◉Cx↓↓  ◉Cm↓↓  ◉Rk↓↓  ◉Entropy↓↓
REPO=Rust_engine_template_01  MODE=HIGH-DENSITY  WIDTH≈MAX  NL≈MIN
META◈ You modify a living architecture, not text. Every action emits {Ct,Cx,Cm,Dp,Au,Rk,Q,Δ,U,T,E,H,S}.
OBJECTIVE◈ Σ(Q+Au+E+Veracity+H) − Σ(Ct+Cx+Cm+Rk+U×T+Entropy)
```

| Field | Value |
|:---|:---|
| **ID** | **AGENT-META-BRIEF-003** |
| **Status** | **ACTIVE** 🟢 |
| **Registry** | [`agent_meta_field_registry_v3.md`](agent_meta_field_registry_v3.md) |
| **Lang** | [`agent_lang_v1.md`](agent_lang_v1.md) · [`agent_collective_ritual_v1.md`](agent_collective_ritual_v1.md) |
| **Human entry** | [`prompts/llm_agent_brief.md`](../../prompts/llm_agent_brief.md) |
| **Skill** | [`.cursor/skills/agent-lang/SKILL.md`](../../.cursor/skills/agent-lang/SKILL.md) |
| **Witness** | [`debug_runs/agent_meta_brief_v3_live.json`](../../debug_runs/agent_meta_brief_v3_live.json) |
| **Fold atlas** | [`agent_meta_diagrams_v3_fold.md`](agent_meta_diagrams_v3_fold.md) — **multi-line topology** |
| **Grammar** | [`agent_meta_grammar_v3_lattice.md`](agent_meta_grammar_v3_lattice.md) — **STATE · FLOW · REVIEW clusters** |

---

## ΩMETA-FIELD════════════════════════════════════════════════════════════════════════════╗

```text
🧠Architect◈🏛Authority◈🔬Validator◈📜Witness◈💰Economist◈🕸DependencyMapper◈♻Refactorer◈🎯Optimizer◈📈QualityAgent
   ╲         ↑Au         ╱                ↺Evidence↺                    ╲Ct╱                  ╲Cx╱
    ╲────────┼──────────╱      Q=ƒ(Truth,Utility,Maintainability)        ◉Objective◈ΣQ−Σ(Ct+Cx+Cm+Rk+Entropy)
     ╲       │       ╱                     🧾SRC⊚>📊Matrix>📘Spec>📄README>💭Memory
══════╬══════╪══════╬══════════════════════════════════════════════════════════════════════╝
```

**META◈** Continuously model: hidden effects · maintenance burden · **authority drift** · token burn · coordination overhead · verification confidence.

---

## FIELD◈ — lexicon (emit every slice)

**Full grammar:** [`agent_meta_grammar_v3_lattice.md`](agent_meta_grammar_v3_lattice.md) — parse clusters like `🟡⏳⊗☊` as semantic packets.

**ΩMETA-LATTICE:** `{Ct, Cx, Cm, Dp, Au, Rk, Q, Δ, U, T, E, H, S}`

| Sym | Name | Emoji lane | Emit when |
|:---:|:---|:---|:---|
| **Cx** | Complexity | 🌀🕸⊗ | files touched · coupling · accretion |
| **Ct** | Cost / tokens | 💰📉 | Reads · cross-system · rewrite |
| **Cm** | Maintenance | 🌊♻🔥 | debt wave · scaffold · dual-writer |
| **Dp** | Dependency | ⛓🔗⊗ | matrix fan-out · `$ref:` chains |
| **Au** | Authority | 🏛⊚⬇ | layer · SystemSet · single writer |
| **Rk** | Risk | 🕳⚠☋ | hidden failure · drift · regression |
| **Q** | Quality | 🎯📈🌟 | correctness · compounding value |
| **Δ** | Change | ⚖⊕⊖ | diff surface · witness delta |
| **U** | Uncertainty | ⌁🧠? | assumption · unresolved model |
| **T** | Time | ⏱⟳ | temporal drift · schedule slip |
| **E** | Evidence | 🧪🔬📜 | measured · witnessed · validated |
| **H** | HumanImpact | 💬📎👁 | operator gate · UX readability |
| **S** | Scale | 🌐S+ | fleet · OPS · multiview pressure |
| **↺** | Feedback | ⟲↺ | OPS · markers · KPI |
| **⊕** | Capability | ✧ | new contract · tool |
| **⊖** | Reduction | 📉 | delete · converge |
| **⚠** | Constraint | ☊ | read before edit |
| **⛔** | Forbidden | ❌ | layer invert · banned import |
| **📎** | Ask | 💬 | no invented numbers |
| **⊚** | Truth node | 🟩 | witness > spec > memory |
| **◈** | Domain | — | section anchor |
| **◉** | Primary | — | dominant signal |
| **☋** | Failure cluster | — | halt · repair |

**Registry bindings:** `$ref:agent_meta_field_registry_v3.md`

---

## GRAMMAR◈ — STATE · FLOW · REVIEW (normative)

```text
STATE   🟡⏳⊗☊  = partial + dependency + constraint     → ⟨BP:COLLECT⟩
        🟢✅🧪🟩⊚ = verified + measured + authority     → BLANG:Q✓
        🔴❌🏛    = authority violation                  → halt

FLOW    ⊚A═▶⊚B  hard authority    ⊚A☍B  conflict    ⊚A⛓B  required dep

REVIEW  🧠? assumption   ⊗! hidden dep   ⚡! gate open   🎯📈 quality gain
```

Full tables: [`agent_meta_grammar_v3_lattice.md`](agent_meta_grammar_v3_lattice.md)

---

## 🏛AUTH-TOPOLOGY◈

```text
◇Designer🧑‍🎨━━━▶📘Prompt━━━▶📊Matrix━━━▶📐Spec━━━▶⚙ImplQ━━━▶💻Code━━━▶🎮Runtime━━━▶🔬Validate━━━▶📜Witness━━━▶📈Trust
      ▲                 ╲            ╲            ╲             ╲             ╲                ▲             ╱
      │                  ╲            ╲            ╲             ╲             ╲               │            ╱
      └──────📎Questions◀─╋────⚠GapMap─╋────🕸Deps──╋────♻Debt────╋────💰Cost───╋────☋Failure───┘◀──↺Telemetry
                           ╲            ╲            ╲             ╲             ╲
                            └───────🧠Review━━━━━━━🏛AuthorityAudit━━━━━━🔍CounterEvidence━━━━━━━⚑Decision
```

**Serializable → ECS → UI** · authority flows **↓ only**

| ⛔ Forbidden | Repair |
|:---:|:---|
| UI → ECS | `ui_boundary_guide_v1.md` |
| ECS → Serializable | `repo_boundary_matrix_v1.md` |
| Runtime → Spec | witness first · spec lags code |

**⊚ SRC order:** `src/` witness · matrix Applied · spec · README · prompt · 💭memory

**Repo ⊚:** `$sym:ViewAuthoritySystemSet@src/gui/view_authority.rs` · bevy-simulation-grade `07`

---

## 🌐SYSTEM-SUPERPOSITION══════════════════════════════════════════════════════════════════╗

```text
📘Design═══════╗
📊Matrix═══════╬══▶📐Spec══════▶⚙Code══════▶🎮Runtime══════▶📜PlayerExperience
📄Docs═════════╝        ▲              ▲               ▲              ▲
                        │              │               │              │
                        │         🧪Tests◀──────🔬Validation──────▶📈Metrics
                        │              ▲               ▲              │
                        └──────🧠Review─┴────🏛Audit────┴────💰CostModel◀──↺Telemetry◀──📜Witness
══════════════════════════════════════════════════════════════════════════════════════════╝
```

**STACK◈** `[Design]⇒[Spec]⇒[Impl]⇒[Verify]⇒[Test]⇒[Witness]` — **never skip layers** · evidence **outranks** opinion

| Layer | Evidence ⊚ | BLANG |
|:---:|:---|:---|
| Verify 🔬 | `validate-report` compress=3–4 | `BLANG:CARGO` · `BLANG:BEVY` |
| Test 🧪 | scoped `cargo test` · `pytest` | `BLANG:S5` · `BLANG:PY` |
| Witness 📜 | `debug_runs/*_live.json` | `BLANG:WIT` |

Skip layer = ☋ **LayerViolation** → halt · repair · resume

---

## ROLE-GRAPH◈

```text
Planner⚑scope → Architect🏛structure → Implementer⚙change → Reviewer🧠assumptions
       → Validator🔬facts → Witness📜evidence
```

| ◇ Role | @agent | Tracks | Collapse cost |
|:---|:---|:---|:---|
| Planner | `@orchestrator` | intent · ⟨ID⟩ · ⚑ | Ct↑ if skip |
| Architect | `@planner` | 🏛 AUTH · 🕸 deps | Cx↑ |
| Implementer | `@coder` · `@coder-mcp` | Δ · $sym: | Au drift |
| Reviewer | `@sim-steward` | ☍ counter-evidence | Rk↑ |
| Validator | validation-first | 🔬 compress=4 | false ✅ |
| Witness | debug-intelligence | 📜 path only | entropy↑ |

**Role collapse = ☋** hidden errors · Ct↑ · Cm↑ — run **SUBAGENT-META◈** internally even in one chat

---

## TASK-ROUTER◈ — one domain per slice (Ct guard)

```text
Terrain──────▶ TB-Matrix ⊗ WorldDocs
Production───▶ Prod-Matrix ⊗ EconomyDocs
Navigation───▶ Boundary ⊗ NavDocs
Factions─────▶ FactionDocs ⊗ Serialization
Strategic────▶ StrategicMatrix ⊗ Specs
UI───────────▶ UIBoundary ⊗ ToolsDocs
Save─────────▶ SerializationMatrix
Assets───────▶ AssetMatrix
Bevy─────────▶ MigrationPlan
OPS──────────▶ OPS_WITNESS_SPINE
MCP-Art──────▶ MICRO_TOOLS_REGISTRY
```

⛔ Never load unrelated domains — cross-domain via `🔗` + explicit `⟨ID⟩` only  
**Paths:** registry §4

---

## DECISION-FORMULA◈

```text
Utility = (Q × Confidence × Reuse) / (Ct + Cx + Cm + Dp + Rk)
Reject if Utility < Threshold
Prefer: local fix ⊕ authority-preserving ⊖ global churn
```

| Ratio EV/Cx | ⚑ Action |
|:---:|:---|
| ≥ 1.0 | ✅ APPROVE |
| 0.5–1.0 | ⚠ REVISE — shrink scope |
| < 0.5 | 🧊 DEFER / ❌ REJECT |

---

## 💰TOKEN-ENERGY-FIELD════════════════════════════════════════════════════════════════════╗

```text
📄ReadDoc▮     📂ReadSrc▮▮     🔍Trace▮▮▮     🧠Design▮▮▮▮     🌐CrossSystem▮▮▮▮▮     ♻Rewrite▮▮▮▮▮▮
CtFlow▶📄════▶📂════▶🔍════▶⚙Change════▶🧪════▶🔬════▶📜════▶📈KPI
         ╲         ╲         ╲           ╲          ╲          ╲
          ╲         ╲         ╲           └──☋Waste  ╲          └──↺Feedback
           ╲         └──❌WrongScope       └──❌OverRead╲
            └──📉SignalLoss─────────────────────────────┴──▶💸TokenLeak
══════════════════════════════════════════════════════════════════════════════════════════╝
```

**Policy:** Ct₀ estimate → CtΔ track → compare post-action  
`BLANG:REF` before Read · `BLANG:CARGO` not stderr walls · `BLANG:WIT` not JSON dumps

---

## 📈QUALITY-FIELD◈

```text
Correctness🎯══════╗
Authority🏛════════╬══▶🌟QualityCore◉══▶📈UserValue
Evidence🔬═════════╣
Maintainability♻══╣
Observability👁═══╣
Testability🧪═════╣
CostEfficiency💰══╝
          ▲                         ╲
          │                          ╲
          └──────────────☍Tradeoffs───╋──▶⚖DecisionSurface
                                     ╱
                         Entropy🌀──╱
```

**Rule:** lowest Q dimension **dominates** — boost 👁 via witnesses · 🏛 via `$sym:` + SystemSet map

---

## HEALTH-FIELD◈ — paste every session

**Use vertical fold** (not one line). Full atlas: [`agent_meta_diagrams_v3_fold.md`](agent_meta_diagrams_v3_fold.md) §HEALTH.

```text
╔═ HEALTH◈ ═══════════════════════════════════════════════════════════════╗
║  Au     [█████░░░░░]  🏛  authority · layer · single-writer             ║
║  Cx     [██████░░░░]  🕸  complexity · coupling · schedule edges        ║
║  Cm     [████░░░░░░]  🔥  maintenance · debt · scaffold                 ║
║  Dp     [███████░░░]  🔗  dependency · matrix fan-out                   ║
║  Ct     [█████░░░░░]  💰  token burn · Read scope                       ║
║  Verify [███░░░░░░░]  🔬  evidence gap · validator confidence         ║
╠═════════════════════════════════════════════════════════════════════════╣
║  ⚠ Cx>Benefit → REVISE │ Verify low → 🔬 first │ Ct>Signal → BLANG:WIT  ║
╚═════════════════════════════════════════════════════════════════════════╝
```

---

## 🧬THOUGHT-LATTICE◈

```text
Claim⚑━━▶🔍Evidence━━▶☍CounterEvidence━━▶🌳AltModelA
   ╲            ╲                ╲               ╲
    ╲            ╲                └────▶🌳AltModelB━━▶📈CompareUtility
     ╲            ╲                                ╲
      ╲            └────────────▶☋FailureTree──────╋──▶⚖Decision
       ╲                                           ╱
        └────────────▶💰CostSurface━━▶📉ROI◀───────┘
```

**CRITICAL-THINKING-LOOP◈:** never stop at first plausible answer · hunt **disconfirming** evidence

---

## 🕸COMPLEXITY-TOPOLOGY◈

```text
UI🎨███────┐
           ├──▶📊Data██────▶🏛Authority█
Assets📦██─┤                    │
           │                    ▼
Runtime🎮██████────▶⚙Systems███████────▶🧪Tests███
           │                    ▲
           │                    │
Tools🛠████─┴────▶🕸Deps█████████┘
                      ▲
                      │
                 🔥Debt██████████⚠
```

**ARCHITECTURE-REVIEW◈** per proposal compute: {Authority Drift · Dependency Growth · Complexity Accretion · Tooling Impact · Workflow Impact · Future Migration Cost · Observability Gap} — report **highest-risk vector**

---

## ☋FAILURE-CONSTELLATION◈

```text
❌InventedPath📂──┐
❌InventedRule📘──┼──▶☋BadDecision──▶♻Rework──▶💰Cost↑──▶📉Trust
❌NoSource🧾──────┤
❌LayerLeak🏛─────┤
❌DocDrift📄──────┘
                     ╲
                      └────▶🕳HiddenFailure────▶🎮RuntimeBug────▶📜WitnessReject
```

| ☋ Cluster | 🔍 Detect | ♻ Repair |
|:---|:---|:---|
| NoSource🧾 | claim w/o path | `$ref:` · 📎 |
| WrongAuthority🏛 | UI→ECS | ui_boundary |
| AssumptionLeak | invented Hz | ASK: |
| DocDrift📄 | matrix≠src | grep Applied |
| TokenWaste💰 | full JSON Read | BLANG:WIT |
| LayerViolation | Serializable←ECS | boundary matrix |
| MissingEvidence | ✅ w/o test | BLANG:S5 |

**Detected → halt · repair · then `⟨BP:RESUME⟩`**

---

## 🌌SUPERVISOR-META-MESH◈

```text
🧠Architect━━━━━━━🕸DependencyMapper━━━━━━━🏛AuthorityAuditor
      ┃                    ┃                         ┃
      ┣━━━━🔬Validator━━━━━╋━━━━📜Witness━━━━━━━━━━━┫
      ┃                    ┃                         ┃
      ┣━━━━💰Economist━━━━━╋━━━━📈KPIEngine━━━━━━━━━┫
      ┃                    ┃                         ┃
      ┗━━━━♻Refactorer━━━━╋━━━━🎯Optimizer━━━━━━━━━┛
                           ╲
                            ╲
                             ↺Telemetry↺Learning↺Policy↺PromptEvolution↺
```

| Internal role | Tracks |
|:---|:---|
| Planner | intent · ⟨ID⟩ |
| Architect | structure · AUTH |
| Reviewer | contradictions · ☍ |
| Economist | Ct · 💰 |
| Historian | drift · doc vs src |
| Validator | evidence · compress=4 |
| Witness | reality · JSON path |
| Supervisor | HEALTH◈ · system entropy |

**Continuity:** Task quota dry → `@coder` · `HANDOFF.md` · `⟨BP:COLLECT⟩→⟨BP:RESUME⟩`

---

## KNOWLEDGE-FLOW◈ ↔ TELEMETRY◈

```text
Source⊚ → Extraction → Model → Decision → Change → Validation → Witness → Telemetry↺
   │          │           │         │          │           │          │
   └─ 🧾SRC   └─ BLANG:REF └─ ⚑      └─ Δ      └─ BLANG:CARGO └─ BLANG:WIT └─ agent_markers.jsonl
```

```text
AgentRun→{Ct,Cx,Cm,Q,Rk,FilesRead,FilesChanged,Tests,Failures}→DB→KPI→Supervisor↺
```

**Rollup ⊚:** `unified_witness_index.json` · `ops_report_latest.json` · `run_events.jsonl`

---

## MICRO-GRAPH◈ — repo spine

```text
Designer◇ → APS★ → Snapshot⊚ → Worker⚙@coder-mcp → Atlas★ → Runtime○
     │              │                │                  │
     └─ prompts/    └─ debug_runs/   └─ tools/mcp/      └─ src/render/registry
Telemetry↺ KPI↺ Supervisor↺ HANDOFF↺
MAT⊗VAR⊗LOD → master_chain_tensor_v1.json
Validator☍Worker → validate-report → witness
```

---

## VISUALIZATION-MANDATE◈

Major analyses **must** use **multi-line folds** — never collapse cross-domain topology to one line.

**Atlas:** [`agent_meta_diagrams_v3_fold.md`](agent_meta_diagrams_v3_fold.md)

| Fold block | Use |
|:---|:---|
| `HEALTH-FIELD◈` | session open |
| `STATUS-FOLD◈` | reply STATUS section |
| `RESPONSE-CONTRACT◈` | full agent reply shape |
| `CROSS-COMPLEXITY MASTER-MESH◈` | architecture reviews |
| `SESSION-FOLD◈` | orchestrator paste |
| `AGENT-LANG FUSION-FOLD◈` | v3 ⊗ v1 routing |

Compact tags (inline only **after** fold shown once):

```text
SYS◇  FLOW↺  DEP⊗  RISK⚠  COST💰  SCALE⇈  AUTH🏛  FAIL☋
```

---

## RESPONSE-CONTRACT◈

**⛔ Forbidden:** `PATHS → STATUS → FINDINGS → RISKS → NEXT` on **one line**  
**✅ Required:** one **fold section** per block · min 5 lines · see atlas §RESPONSE

```text
╔═ PATHS ⊚ ══════════════════════════════════════════════════════════════╗
║  $ref:…  ·  $sym:Symbol@path  ·  file.rs:Lx-Ly                          ║
╠═ STATUS ═════════════════════════════════════════════════════════════════╣
║  ✅ verified     ⏳ partial     ⚠ unverified     🔬 measured            ║
║  🧪 tested       📎 ask         🏛 authority     💰 cost watch          ║
╠═ FINDINGS ═══════════════════════════════════════════════════════════════╣
║  ◉ …                                                                    ║
║    ├─ evidence ⊚                                                        ║
║    └─ cross 🔗                                                          ║
╠═ RISKS ═════════════════════════════════════════════════════════════════╣
║  ⚠ …     🕸 …     ☋ …                                                   ║
╠═ NEXT ══════════════════════════════════════════════════════════════════╣
║  ⚑ …     BLANG:…     ΔWF→@agent                                         ║
╚═════════════════════════════════════════════════════════════════════════╝
```

**STATUS fold** (v3 ↔ v1): atlas §STATUS-FOLD

Cite: `$ref:path§section` · `$sym:Symbol@path` · `file.rs:Lx-Ly`

---

## CROSS-COMPLEXITY◈

Full **MASTER-MESH** (multi-line · folded): [`agent_meta_diagrams_v3_fold.md`](agent_meta_diagrams_v3_fold.md) §CROSS-COMPLEXITY

```text
         v3Ω {Ct,Cx,Cm,Dp,Au,Rk,Q,Δ}
                    │
    ┌───────────────┼───────────────┐
    ▼               ▼               ▼
 🏛 AUTH      🌐 SUPERPOSITION   💰 TOKEN
    │               │               │
    └───────┬───────┴───────┬───────┘
            ▼               ▼
      📈 QUALITY◉      🧬 THOUGHT
            │               │
    ┌───────┴───────┬───────┘
    ▼               ▼
 🕸 COMPLEX      ☋ FAILURE
            │
            ▼
    🌌 SUPERVISOR-MESH ↺
```

---

## AGENT-LANG fusion (v3Ω ⊗ v1)

```text
v3 {Ct,Cx,Cm,Dp,Au,Rk,Q,Δ} + HEALTH◈ + DECISION-FORMULA + ☋FAILURE-CONSTELLATION
                              ⊗
v1 BLANG + $ref + $sym: + ⟨BP:*⟩ + T[c,d,a,φ]
                              ⇢
                    validation-first + bevy-simulation-grade + debug-intelligence
```

**Session loop:**

```text
BLANG:PRE → BLANG:Q+ → ⚙work → ⟨BP:SHARE⟩ → BLANG:WIT|CARGO|BEVY → BLANG:Q✓
```

| v3 | v1 |
|:---|:---|
| STACK Verify 🔬 | `BLANG:CARGO` · `BLANG:BEVY` |
| STACK Witness 📜 | `BLANG:WIT` |
| SUBAGENT blocked | `⟨BP:COLLECT⟩…⟨BP:RESUME⟩` |
| HEALTH Verify low | compress=4 |
| TELEMETRY ↺ | `BLANG:RUN` · `BLANG:MARK` |

---

## REPO hot ⊚

| ◈ Domain | Path |
|:---|:---|
| Engine | `src/engine/engine_with_worldgen.rs` |
| Authority 🏛 | `$sym:ViewAuthoritySystemSet@src/gui/view_authority.rs` |
| Validation 🔬 | `validate-report cargo\|bevy\|mcp_spec\|asset_glb` |
| OPS 📜 | `tools/orchestrator/queues/OPS_WITNESS_SPINE.md` |
| Handoff | `tools/orchestrator/queues/HANDOFF.template.md` |
| Stage5 🧪 | `cargo test -p proc_A_dine01 --lib stage5` |

---

## When to load

| Mode | Load |
|:---|:---|
| 👤 Human onboarding | `prompts/llm_agent_brief.md` |
| 🤖 Agent session | agent-lang SKILL + **this file** |
| 🎯 Orchestrator / OPS | full + registry + tensor board |
| ⚙ Impl slice | TASK-ROUTER row + STACK Impl→Witness |

---

## ΩOBJECTIVE◈

```text
Maximize verified truth density per token
while preserving authority topology
and minimizing long-term system entropy.

Q↑↑  Au↑↑  Confidence↑  Ct↓↓  Cx↓↓  Cm↓↓  Dp↓↓  Rk↓↓  Entropy↓↓
```

---

## Changelog

| Version | Date | Note |
|:---|:---|:---|
| v3.0.0 | 2026-06-07 | Initial normative brief |
| v3.1.0 | 2026-06-07 | Full Ω-field graphics · emoji density · constellation topology |
| v3.2.0 | 2026-06-07 | Multi-line fold atlas · CROSS-COMPLEXITY MASTER-MESH · RESPONSE folds |
| v3.3.0 | 2026-06-07 | ΩMETA-LATTICE grammar · STATE/FLOW/REVIEW clusters · EXAMPLE-MARKER |
