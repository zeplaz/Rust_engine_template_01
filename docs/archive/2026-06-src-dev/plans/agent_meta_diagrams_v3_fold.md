# AGENT-META diagrams `v3Ω` — multi-line fold atlas

| Field | Value |
|:---|:---|
| **ID** | **AGENT-META-DIAGRAMS-003-FOLD** |
| **Parent** | [`agent_meta_brief_v3.md`](agent_meta_brief_v3.md) |
| **Rule** | **Never collapse topology to one line** — use folds below |

---

## FOLD-KEY◈ — how to read

```text
┌─ FOLD:OPEN  ─ section expands vertically (preferred in replies)
├─ FOLD:CROSS ─ diagonal link between two domains
└─ FOLD:CLOSE ─ summary bar only when Ct critical
```

---

## HEALTH-FIELD◈ — vertical paste (session open)

```text
╔═ HEALTH◈ ═══════════════════════════════════════════════════════════════╗
║  Au     [█████░░░░░]  🏛  authority · layer · single-writer             ║
║  Cx     [██████░░░░]  🕸  complexity · coupling · schedule edges        ║
║  Cm     [████░░░░░░]  🔥  maintenance · debt · scaffold                 ║
║  Dp     [███████░░░]  🔗  dependency · matrix fan-out · $ref chains     ║
║  Ct     [█████░░░░░]  💰  token burn · Read scope · rewrite cost        ║
║  Verify [███░░░░░░░]  🔬  evidence gap · validator confidence         ║
╠═════════════════════════════════════════════════════════════════════════╣
║  ⚠ FLAGS                                                                ║
║    Cx > Benefit  ──▶ ⚑ REVISE scope                                     ║
║    Cm > Value    ──▶ ♻ complete · don't delete blind                    ║
║    Dp > Need     ──▶ drop unrelated matrix loads                        ║
║    Ct > Signal   ──▶ BLANG:WIT · not raw JSON/stderr                    ║
║    Verify low    ──▶ 🔬 validate before next edit                       ║
╚═════════════════════════════════════════════════════════════════════════╝
```

---

## STATUS-FOLD◈ — v3 ↔ v1 ↔ emoji (reply lane)

```text
┌─────────────────────────────────────────────────────────────────────────┐
│ STATUS-FOLD◈                                                            │
├──────────────┬──────────────┬───────────────────────────────────────────┤
│ v3 reply     │ v1 marker    │ when                                      │
├──────────────┼──────────────┼───────────────────────────────────────────┤
│ ✅ verified  │ 🟢           │ witness green · validator pass            │
│ ⏳ partial   │ 🟡           │ qualified · incomplete                    │
│ ⚠ unverified │ 🟡 + note    │ needs BLANG:CARGO / BLANG:WIT             │
│ 📎 ask       │ 💬           │ human gate · no invented Hz/caps          │
│ ❌ forbidden │ 🔴 + ⛔       │ banned import · layer invert              │
│ ♻ candidate  │ 🧊           │ refactor ticket · defer                   │
│ 🔬 measure   │ BLANG:CARGO  │ validation-first compress=4               │
│ 🧪 tests     │ BLANG:S5/PY  │ stage5 · pytest scoped                    │
│ 🔥 debt      │ 🧊 + note    │ cleanup-intelligence classify             │
│ 🏛 authority │ AUTH spine   │ $sym:ViewAuthoritySystemSet@…             │
│ 🕸 dependency│ 🧩           │ matrix cross-ref · one domain/slice       │
│ 💰 cost      │ Ct high      │ shrink Read · BLANG:REF first              │
│ 📈 gain      │ Q↑           │ ship witness · closure                    │
│ 🚦 gate      │ ⚡P0         │ validate_p0_gate_plain                    │
└──────────────┴──────────────┴───────────────────────────────────────────┘
```

---

## RESPONSE-CONTRACT◈ — multi-line reply template (mandatory shape)

```text
╔═ PATHS ⊚ ══════════════════════════════════════════════════════════════╗
║  $ref:…  ·  $sym:Symbol@path  ·  file.rs:Lx-Ly                          ║
╠═ STATUS ═════════════════════════════════════════════════════════════════╣
║  ✅ … verified          ⏳ … partial          ⚠ … unverified            ║
║  🔬 … measured            🧪 … tested           📎 … ask if blocked       ║
╠═ FINDINGS ═══════════════════════════════════════════════════════════════╣
║  ◉ primary signal (one line)                                            ║
║    ├─ evidence ⊚                                                        ║
║    └─ cross-link 🔗                                                     ║
╠═ RISKS ═════════════════════════════════════════════════════════════════╣
║  ⚠ …                    🕸 …                    ☋ … if halt             ║
╠═ NEXT ══════════════════════════════════════════════════════════════════╣
║  ⚑ …                    BLANG:…               ΔWF→@agent                ║
╚═════════════════════════════════════════════════════════════════════════╝
```

**⛔ Forbidden:** single-line dump of PATHS+STATUS+FINDINGS+RISKS+NEXT  
**✅ Required:** one **fold section** per contract block · min 5 lines

---

## CROSS-COMPLEXITY MASTER-MESH◈

```text
                    ┌─────────── ΩMETA-FIELD ───────────┐
                    │  🧠 Architect    🏛 Authority       │
                    │  🔬 Validator    📜 Witness       │
                    │  💰 Economist    📈 Quality       │
                    └───────────────┬───────────────────┘
                                    │ emits {Ct,Cx,Cm,Dp,Au,Rk,Q,Δ}
          ┌─────────────────────────┼─────────────────────────┐
          │                         │                         │
          ▼                         ▼                         ▼
   ┌─────────────┐           ┌─────────────┐           ┌─────────────┐
   │ 🏛 AUTH     │◀─ FOLD ──▶│ 🌐 SUPER    │── FOLD ─▶│ 💰 TOKEN    │
   │  TOPOLOGY   │   CROSS    │  POSITION   │   CROSS  │  ENERGY     │
   │             │            │             │          │             │
   │ ◇Designer   │            │ 📘 Design   │          │ 📄 ReadDoc ▮ │
   │  ↓          │            │  ↓          │          │ 📂 ReadSrc▮▮│
   │ 📜 Witness  │            │ 📜 Witness  │          │ 🔍 Trace ▮▮▮│
   └──────┬──────┘            └──────┬──────┘          └──────┬──────┘
          │                         │                         │
          │    ┌────────────────────┼────────────────────┐    │
          │    │                    │                    │    │
          ▼    ▼                    ▼                    ▼    ▼
   ┌─────────────────────────────────────────────────────────────┐
   │              📈 QUALITY-FIELD  ◉ QualityCore                │
   │   Correctness🎯 ──┐                                         │
   │   Authority🏛  ───┼──▶ 🌟 ──▶ 📈 UserValue                  │
   │   Evidence🔬   ───┤         ▲                               │
   │   Observability👁 ─┘         │                               │
   │                    ☍ Tradeoffs ──▶ ⚖ Decision               │
   └──────────────────────────┬──────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
          ▼                   ▼                   ▼
   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
   │ 🧬 THOUGHT  │     │ 🕸 COMPLEX  │     │ ☋ FAILURE   │
   │   LATTICE   │     │  TOPOLOGY   │     │ CONSTELL    │
   │             │     │             │     │             │
   │ Claim⚑      │     │ UI🎨 ──┐    │     │ ❌ NoSource │
   │  ↓          │     │ Assets📦┤   │     │ ❌ LayerLeak│
   │ 🔍 Evidence │     │ Runtime🎮──▶│     │     ↓       │
   │  ↓          │     │ Tools🛠 ──┘   │     │ ☋ BadDec  │
   │ ☍ Counter   │     │     ↓         │     │     ↓     │
   │  ↓          │     │ 🔥 Debt⚠    │     │ 📉 Trust  │
   │ ⚖ Decision  │     └─────────────┘     └─────────────┘
   └──────┬──────┘
          │
          ▼
   ┌─────────────────────────────────────────────────────────────┐
   │           🌌 SUPERVISOR-META-MESH  ↺ Telemetry              │
   │                                                             │
   │   🧠 Architect ──── 🕸 DepMapper ──── 🏛 AuthAuditor        │
   │        ┃                 ┃                  ┃               │
   │   🔬 Validator ─── 📜 Witness ─── 💰 Economist             │
   │        ┃                 ┃                  ┃               │
   │   ♻ Refactorer ─── 🎯 Optimizer ─── 📈 KPIEngine          │
   │                        ↺                                  │
   │              Learning · Policy · PromptEvolution          │
   └─────────────────────────────────────────────────────────────┘
```

---

## SESSION-FOLD◈ — orchestrator paste (multi-line)

```text
LLM◆BRIEF:v3Ω
  ◉Q↑↑  ◉Au↑↑  ◉Veracity↑↑
  ◉Ct↓↓  ◉Cx↓↓  ◉Cm↓↓  ◉Rk↓↓  ◉Entropy↓↓
REPO=Rust_engine_template_01
MODE=HIGH-DENSITY
⟨ID⟩=…

HEALTH◈
  Au     [█████░░░░░]
  Cx     [██████░░░░]
  Cm     [████░░░░░░]
  Dp     [███████░░░]
  Ct     [█████░░░░░]
  Verify [███░░░░░░░]

LOOP◈
  BLANG:PRE
    → BLANG:Q+
    → ⚙ work
    → ⟨BP:SHARE⟩
    → BLANG:WIT | CARGO | BEVY
    → BLANG:Q✓

REPLY◈ PATHS → STATUS → FINDINGS → RISKS → NEXT  (use FOLD sections)
```

---

## AGENT-LANG FUSION-FOLD◈

```text
         v3Ω LAYER                          v1 LAYER
  ┌─────────────────────┐            ┌─────────────────────┐
  │ {Ct,Cx,Cm,Dp,       │            │ BLANG:PRE/Q+/Q✓     │
  │  Au,Rk,Q,Δ}         │   ⊗ fuse   │ $ref: · $sym:       │
  │ HEALTH◈             │◀──────────▶│ ⟨BP:COLLECT…RESUME⟩ │
  │ DECISION-FORMULA    │            │ T[c,d,a,φ]          │
  │ ☋ FAILURE-CONSTELL  │            │ agent_markers.jsonl │
  └──────────┬──────────┘            └──────────┬──────────┘
             │                                  │
             └──────────────┬───────────────────┘
                            ▼
              ┌─────────────────────────────┐
              │  SKILLS (attach pairs)      │
              │  validation-first    🔬     │
              │  bevy-simulation-grade 🏛   │
              │  debug-intelligence  📜     │
              │  operations-intelligence ↺  │
              └─────────────────────────────┘
```

---

## GRAMMAR-LATTICE◈ — normative source

Full spec: [`agent_meta_grammar_v3_lattice.md`](agent_meta_grammar_v3_lattice.md)

```text
FIELD◈ΩMETA-LATTICE  {Ct,Cx,Cm,Dp,Au,Rk,Q,Δ,U,T,E,H,S}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 STATE-GRAMMAR◈
 🟢✅⊚   verified+authority    🟡⏳⊗☊  partial·dep·constraint   🔴❌🏛  auth violation
 🟢✅🧪   measured              ⚡🚦📜  witness gate open
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 FLOW-GRAMMAR◈
 ⊚A═▶⊚B  hard auth   ⊚A☍B  conflict   ⊚A⛓B  required   ⊚A⌁B  uncertain
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 REVIEW-TAGS◈
 🧠?  ☍!  ⌁?  ⊗!  🕳!  💰!  🏛!  🌀!  🌊!  ⚡!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## Changelog

| Version | Date | Note |
|:---|:---|:---|
| v3.2.0 | 2026-06-07 | Multi-line fold atlas · CROSS-COMPLEXITY MASTER-MESH |
| v3.3.0 | 2026-06-07 | Grammar-lattice cross-link |
