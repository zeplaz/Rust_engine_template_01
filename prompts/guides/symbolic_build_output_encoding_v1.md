# GUIDE-SYMBOLIC-BUILD-OUTPUT-001 — Build / compile → SYMLANG digest `v1`

```text
⟦SYMLANG⟧⟐v1  ◈GUIDE  ◈NORMATIVE
⟨ID⟩ GUIDE-SYMBOLIC-BUILD-OUTPUT-001
Parent: $ref:src/dev/plan_vfx_spectator_scale_program_v1.md
Program: VSS-T5-001
Status: ACTIVE · 2026-08-28
```

**Rule:** Agents MUST NOT reason on raw `cargo check` / `cargo test` stderr. Pump output through the encoder → consume `SymbolicBuildDigest` or `ValidationReport` (compression 3–4).

---

## Pipeline

```text
cargo check --message-format=json
        │
        ▼
validate_cargo_report (existing ValidationReport)
        │
        ▼
symbolic_build_encoder.encode_digest()
        │
        ├── symcodes[]     token-short issue identities
        ├── trip_hits[]    VSS trip registry matches on touched files
        ├── known_fixes[]  human-editable patterns (validators/knowledge.py)
        └── handoff        MCP tool JSON for subagents
```

---

## Symbolic issue code (symcode) format

```text
⟨SEV⟩◈⟨KIND⟩◈⟨PATH⟩:⟨LINE⟩◈⟨RUSTC⟩◈⟨SIG8⟩
```

| Field | Example |
|:---|:---|
| SEV | `ERR` `WRN` `INF` |
| KIND | `TypeMismatch` `BorrowIssue` `MissingImport` |
| PATH | `src/engine/test_harness.rs` |
| LINE | `1769` |
| RUSTC | `E0308` or `-` |
| SIG8 | first 8 hex of stable hash(message+location) |

Example:

```text
ERR◈TypeMismatch◈src/engine/test_harness.rs:1769◈E0308◈a3f2c91b
```

Agents grep symcodes across sessions — **human improvable** via `tools/mcp/python/rust_engine_mcp/validators/knowledge.py` known_fixes.

---

## SymbolicBuildDigest JSON schema

```json
{
  "schema": "symbolic_build_digest_v1",
  "digest_id": "sha256-prefix",
  "source": "cargo_check",
  "status": "passed|failed|warning",
  "compression_level": 3,
  "summary": "cargo check: 0 errors, 7 warnings",
  "symcodes": ["ERR◈..."],
  "trip_hits": ["TRIP-VSS-HARNESS"],
  "touched_files": ["src/engine/test_harness.rs"],
  "validation_report": { },
  "human_notes_path": "debug_runs/symbolic_digests/{digest_id}.notes.md"
}
```

**human_notes_path:** Optional markdown operators edit — encoder preserves on re-run if file exists.

---

## MCP / CLI usage

| Tool | When |
|:---|:---|
| `symbolic_build_digest_tool` | After any agent `cargo check` / `cargo test` |
| `validate_cargo_report` | Still authoritative for pass/fail |
| `token_savings_guide` | Before spawning L1/L2 subagents |

```powershell
# CLI (when wired):
python -m rust_engine_mcp.symbolic_build_encoder --package proc_A_dine01 --compress 3
```

---

## Token shortcuts (repeat systems)

| Instead of | Use |
|:---|:---|
| Pasting 200-line cargo wall | `symcodes` top 8 + `known_fixes` |
| Re-explaining fire authority | `TRIP-VSS-FIRE-AUTHORITY` link |
| Re-reading full witness JSON | `witness_brief <path>` |
| Parallel Task on same files | HANDOFF serial matrix in VSS plan |

---

## Subagent handoff

Parent agents attach to Task prompts:

```yaml
symbolic_digest: <path or inline symcodes[]>
trip_hits: [TRIP-VSS-HARNESS]
mandatory_reads:
  - src/dev/TRIP_VSS_TEST_HARNESS.md
dissent_slot: required
```

---

## Human improvement loop

1. Operator edits `debug_runs/symbolic_digests/*.notes.md`
2. Promote recurring fix to `validators/knowledge.py`
3. Ops indexes digests in Postgres (dev-only, VSS-T5-002) for cross-run analytics

**Do not** put sim authority in Postgres — $ref:src/dev/guide_sim_effect_spine_v1.md STORAGE-THREE-WORLDS.
