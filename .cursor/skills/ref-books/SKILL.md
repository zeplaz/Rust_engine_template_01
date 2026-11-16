---
name: ref-books
description: >-
  Route code work through the ref-books index and PLAN-REF-BOOKS-UPGRADE-v2 slice
  cards. Use before algorithm, data-layout, or schedule changes; when picking the
  next OPEN slice; or when looking up a book section for an existing module.
  Triggers: lookup.py, ref books, book section, matrix row, PLAN-REF-BOOKS-UPGRADE-v2.
---

# Ref-books — slice protocol + lookup

**Ledger (authoritative):** `src/dev/ref_books_index/adoption.tsv`  
**Stance / honest exits:** `src/dev/plan_ref_books_upgrade_v3.md` §0.1 — stop and report `BLOCKED:` / `UNWIRED:` when a slice cannot be wired honestly; `ADOPTED` only with a production caller `file:line`.  
**Slice cards:** `src/dev/plan_ref_books_upgrade_v2.md` (§3 cards; §9 is historical seed only)  
**DEFER/PARTIAL unlock:** `src/dev/plan_ref_books_defer_unlock_v1.md`  
**Index:** `src/dev/ref_books_index/` (do not copy matrix rows into this skill)

## §0 protocol (one slice per session)

```text
1. ORIENT   python src/dev/ref_books_index/lookup.py --code <first file>
2. RECON    card Recon; if Done-if → ledger ADOPTED (production caller), commit, STOP
3. READ     lookup.py --sec / --read (≤ ~12 PDF pages)
4. DO       edit only the card's Files
5. ACCEPT   every Accept command must pass
6. LEDGER   update adoption.tsv; commit "[area] <slice id>: <what>"
```

## Lookup commands

```bash
python src/dev/ref_books_index/lookup.py --brief <code path>
python src/dev/ref_books_index/lookup.py --row <ID>
python src/dev/ref_books_index/lookup.py --open [DOMAIN]
python src/dev/ref_books_index/lookup.py --json <terms>
python src/dev/ref_books_index/lookup.py --check
```

CLI parity: `python -m rust_engine_mcp.cli ref-lookup …` from `tools/mcp/python`.