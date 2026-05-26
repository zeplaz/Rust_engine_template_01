# Single worktree policy v1

**Effective:** 2026-05-25  
**Problem:** Gate witnesses and code diverged between `master` (main clone) and `cursor/aa7b9353` (Cursor worktree `ds5i`). Copying `debug_runs/*.json` without merging `src/` caused false PASS reads.

---

## Canonical repo (use this only)

| Item | Value |
|------|--------|
| **Path** | `C:\dev\github\Rust_engine_template_01` |
| **Branch** | `master` |
| **Proof** | `cargo test -p proc_A_dine01 --lib steward_w3_gate_001_lib_bundle` |
| **Gate doc** | [`steward_w3_gate_v1.md`](steward_w3_gate_v1.md) |

All agents, stewards, coders, and visual runs use **this path** only.

---

## Deprecated for product / gate work

| Path | Branch | Status |
|------|--------|--------|
| `C:\Users\oz_\.cursor\worktrees\Rust_engine_template_01\ds5i` | `cursor/aa7b9353` @ `c91d4f4` | **Retire** — scaffold witness writer at base; gate infra on **master** WIP |

Witness JSON was briefly copied **from master → ds5i** for visibility only. That is **not** consolidation.  
**Never** merge `master` into `ds5i` as the primary fix — **master absorbs everything; ds5i is deleted.**

### Flow direction (always)

```text
ds5i / cursor branches  ──(cherry-pick or manual port, if unique)──►  master @ C:\dev\github\Rust_engine_template_01
                                                                      │
                                                                      └── sole source of truth
```

---

## Consolidation (operator) — into `master` only

```powershell
# 1) Work ONLY on master (primary clone)
cd C:\dev\github\Rust_engine_template_01
git checkout master

# 2) If ds5i has unique commits/files you still need, port them HERE (not the reverse):
#    git cherry-pick <sha>   OR   copy specific paths, then commit on master

# 3) Commit all gate WIP on master
git add -A
git commit -m "UI Wave 3 gate: shell 2A/2B, minimap M2, steward bundle"
git push origin master

# 4) Remove ds5i — do not keep a second “primary”
git worktree remove "C:\Users\oz_\.cursor\worktrees\Rust_engine_template_01\ds5i" --force
```

**Cursor:** open folder `C:\dev\github\Rust_engine_template_01` (this agent root was moved here 2026-05-25).

---

## Agent rules

1. Do not treat copied `debug_runs/*.json` as proof when `write_ui_shell_migration_witness()` still emits scaffold fields.
2. Run steward bundle from this path:  
   `set CARGO_TARGET_DIR=%CD%\target\test-alt-steward`  
   `cargo test -p proc_A_dine01 --lib steward_w3_gate_001_lib_bundle`
3. Update queues/ledger only on this path after commits land on `master`.
