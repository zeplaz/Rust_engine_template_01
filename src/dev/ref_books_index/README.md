# Ref-books index v2 — cheap routing from our code to book sections

Built from the **tables of contents** (PDF outlines plus numbered x.y.z headings) of the books in
`C:\dev\ref books\`. No book text is stored here. This replaces nothing; the older
`C:\dev\ref books\lookup\` folder is separate and was not used to build this one.

## Files

| File | What | Size |
|---|---|---|
| [`systems.md`](systems.md) | **Start here.** Game system → niche "gems" + matrix row ids | ~12 systems |
| [`matrix.md`](matrix.md) | Concept → book sections → our code paths → one-line takeaway (`★` niche, `●` core, `○` low) | 217 rows, 129 `★` |
| `toc/<book>.tsv` | Full-depth TOC per book: `sec depth start end title` (PDF page indices) | ~3k rows |
| [`lookup.py`](lookup.py) | Query tool (stdlib only) | — |
| [`build_toc.py`](build_toc.py) | Rebuilds `toc/` from the PDFs (pypdf + pdftotext) | — |

## Books

| id | Book | Use for | Precedence |
|---|---|---|---|
| `gea4v1` | Game Engine Architecture 4e, Vol I (foundations) | loop and time, concurrency, memory, containers, resources, HID, debug | primary |
| `gea4v2` | Game Engine Architecture 4e, Vol II (graphics, motion, sound, gameplay) | render, lighting, physics, audio, object model, updates, events, streaming | primary |
| `aig3` | AI for Games 3e (Millington) | steering, formations, pathfinding, decisions, tactical analysis, PCG, AI scheduling and LOD | primary |
| `gea3` | Game Engine Architecture 3e (single volume) | only when 4e has no matching section | fallback |

## Agent workflow (3 hops, no PDF reading until the last step)

```text
1. systems.md  -> find your game system, pick a gem or row id
2. lookup.py   -> resolve to book + section + PDF page range (+ which code it maps to)
3. --read      -> pull only those pages as text when you actually need the details
```

```bash
python src/dev/ref_books_index/lookup.py fire spread
```

```bash
python src/dev/ref_books_index/lookup.py --code economy/logistics/orders/sim.rs
```

```bash
python src/dev/ref_books_index/lookup.py --sec aig3:6.2
```

```bash
python src/dev/ref_books_index/lookup.py --read aig3:5.7.7
```

```bash
python src/dev/ref_books_index/lookup.py --niche NAV
```

Plain `grep` works too: every matrix row is one line (`grep -i "influence" src/dev/ref_books_index/matrix.md`),
and every TOC entry is one line (`grep -i "erosion" src/dev/ref_books_index/toc/*.tsv`).

## Conventions

- Page numbers are **PDF page indices** (what `pdftotext -f/-l` takes), not printed page numbers. `end` is the next sibling's start page, so it can share a page.
- Matrix code paths are relative to `src/`; paths starting with `tools/` are repo-relative.
- A book idea maps onto an **existing** authority and SystemSet (AGENTS.md: one writer per domain, Sim → View → Render → UI). Reject anything that adds a parallel stack.
- `REF_BOOKS_DIR` overrides the PDF folder.

## Maintenance

```bash
python -I src/dev/ref_books_index/lookup.py --check
```

`--check` fails if a matrix ref is missing from a TOC or a code path no longer exists (run it after moves or renames).
`build_toc.py` regenerates `toc/` if a PDF is replaced; matrix rows reference section ids, so they survive a rebuild.
