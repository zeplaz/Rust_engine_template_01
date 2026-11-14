"""Cheap ref-book lookup for agents. Stdlib only; reads matrix.md + toc/*.tsv (never the PDFs,
except `--read`, which shells out to pdftotext for just the requested pages).

    python src/dev/ref_books_index/lookup.py fire spread            # concept -> book pages + code
    python src/dev/ref_books_index/lookup.py --code combat/relax.rs  # file -> which book sections apply
    python src/dev/ref_books_index/lookup.py --sec gea4v2:17.6.3     # section subtree + page range
    python src/dev/ref_books_index/lookup.py --read aig3:5.7.7       # print that section's text
    python src/dev/ref_books_index/lookup.py --niche [DOMAIN]        # list ★ rows
    python src/dev/ref_books_index/lookup.py --check                 # validate refs + code paths
    python src/dev/ref_books_index/lookup.py --json fire spread      # same as search, JSON array
    python src/dev/ref_books_index/lookup.py --json --code PATH      # --code / --niche also take --json
    python src/dev/ref_books_index/lookup.py --row PHYS-02           # one matrix row + refs
    python src/dev/ref_books_index/lookup.py --brief combat/ballistics.rs  # ≤12 lines: id, best ref, --read
"""
import json, os, re, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
BOOK_DIR = os.environ.get("REF_BOOKS_DIR", r"$REF_BOOKS_DIR")
BOOKS = {
    "gea4v1": ("GEA 4e Vol I", "game-engine-architecture-volume-i-foundations-and-core-engine-systems-4_compress.pdf"),
    "gea4v2": ("GEA 4e Vol II", "Game Engine Architecture Volume II_ Graphics_ Motion_ and Sound_ 4th Edition.sanet.st.pdf"),
    "aig3":   ("AI for Games 3e", "ai-for-games-3e_compress.pdf"),
    "gea3":   ("GEA 3e (fallback)", "Gregory, Jason - Game engine architecture.pdf"),
}
try: sys.stdout.reconfigure(encoding="utf-8")
except Exception: pass

def load_toc():
    toc = {}
    for b in BOOKS:
        rows = {}
        with open(os.path.join(HERE, "toc", f"{b}.tsv"), encoding="utf-8") as f:
            next(f)
            for line in f:
                sec, depth, start, end, title = line.rstrip("\n").split("\t")
                rows[sec] = (int(depth), int(start), int(end), title)
        toc[b] = rows
    return toc

def load_matrix():
    rows = []
    with open(os.path.join(HERE, "matrix.md"), encoding="utf-8") as f:
        for line in f:
            if not re.match(r"^\| [A-Z]+-\d+ ", line): continue
            c = [x.strip() for x in line.strip().strip("|").split("|")]
            rows.append(dict(id=c[0], tier=c[1], concept=c[2], kw=c[3],
                             refs=[r.strip() for r in c[4].split(";") if r.strip()],
                             code=[p.strip() for p in c[5].split(";") if p.strip() and p.strip() != "—"],
                             take=c[6]))
    return rows

def code_path(p):
    return os.path.join(REPO, p) if p.startswith("tools/") else os.path.join(REPO, "src", p)

def ref_line(toc, ref):
    b, _, sec = ref.partition(":")
    r = toc.get(b, {}).get(sec)
    if not r: return f"    {ref:<20} (missing in toc)"
    return f"    {b:<6} {sec:<10} p{r[1]}-{r[2]:<5} {r[3]}"

def ref_obj(toc, ref):
    b, _, sec = ref.partition(":")
    r = toc.get(b, {}).get(sec)
    if not r:
        return {"book": b, "sec": sec, "start": None, "end": None, "title": None}
    return {"book": b, "sec": sec, "start": r[1], "end": r[2], "title": r[3]}

def row_json(toc, r):
    return {
        "id": r["id"],
        "tier": r["tier"],
        "concept": r["concept"],
        "refs": [ref_obj(toc, ref) for ref in r["refs"]],
        "code": list(r["code"]),
        "take": r["take"],
    }

def emit_json(rows, toc):
    print(json.dumps([row_json(toc, r) for r in rows], ensure_ascii=False))

def show_row(toc, r, refs=True):
    print(f"{r['id']} {r['tier']} {r['concept']}")
    if refs:
        for ref in r["refs"]: print(ref_line(toc, ref))
    if r["code"]: print("    code: " + "; ".join(r["code"]))
    print("    -> " + r["take"])

def words(s): return re.findall(r"[a-z0-9*]+", s.lower())

def score_search(terms, mat):
    terms = [t.lower() for t in terms]
    scored = []
    for r in mat:
        hay_kw, hay_c = " ".join([r["kw"], r["concept"]]).lower(), " ".join(r["code"] + [r["take"]]).lower()
        s = sum(3 * (t in hay_kw) + (t in hay_c) for t in terms)
        if all(t in hay_kw + " " + hay_c for t in terms): s += 5
        if s: scored.append((s, r))
    scored.sort(key=lambda x: (-x[0], x[1]["id"]))
    return scored

def search(terms, toc, mat, as_json=False):
    scored = score_search(terms, mat)
    if as_json:
        emit_json([r for _, r in scored[:8]], toc)
        return
    print(f"== matrix ({len(scored)} rows) ==")
    for _, r in scored[:8]: show_row(toc, r)
    hits = []
    for b, rows in toc.items():
        for sec, (d, st, en, title) in rows.items():
            t = title.lower()
            if all(x in t for x in terms): hits.append((b, sec, st, en, title))
    if any(h[0] != "gea3" for h in hits):  # gea3 is fallback only
        hits = [h for h in hits if h[0] != "gea3"]
    print(f"== toc titles ({len(hits)}) ==")
    for b, sec, st, en, title in hits[:20]: print(f"    {b:<6} {sec:<10} p{st}-{en:<5} {title}")

def rows_for_code(frag, mat):
    frag = frag.replace("\\", "/").removeprefix("src/")
    return [r for r in mat if any(frag in p or p.rstrip("/") in frag for p in r["code"])]

def by_code(frag, toc, mat, as_json=False):
    frag = frag.replace("\\", "/").removeprefix("src/")
    rows = rows_for_code(frag, mat)
    if as_json:
        emit_json(rows, toc)
        return
    print(f"== rows touching {frag} ({len(rows)}) ==")
    for r in rows: show_row(toc, r)

def niche_rows(mat, domain=None):
    out = []
    for r in mat:
        if r["tier"] == "★" and (domain is None or r["id"].startswith(domain.upper())):
            out.append(r)
    return out

def by_niche(toc, mat, domain=None, as_json=False):
    rows = niche_rows(mat, domain)
    if as_json:
        emit_json(rows, toc)
        return
    for r in rows:
        show_row(toc, r, refs=False)

def best_ref(toc, r):
    for ref in r["refs"]:
        b, _, sec = ref.partition(":")
        if toc.get(b, {}).get(sec):
            return ref
    return r["refs"][0] if r["refs"] else None

def by_row(row_id, toc, mat, as_json=False):
    want = row_id.upper()
    rows = [r for r in mat if r["id"] == want]
    if not rows:
        sys.exit(f"unknown row {row_id}")
    if as_json:
        emit_json(rows, toc)
        return
    show_row(toc, rows[0], refs=True)

def by_brief(frag, toc, mat):
    rows = rows_for_code(frag, mat)
    lines = 0
    for r in rows:
        if lines >= 12:
            break
        ref = best_ref(toc, r)
        if not ref:
            print(f"{r['id']}  (no refs)")
            lines += 1
            continue
        print(f"{r['id']}  {ref}  →  python src/dev/ref_books_index/lookup.py --read {ref}")
        lines += 1

def section(ref, toc):
    b, _, sec = ref.partition(":")
    rows = toc[b]
    if sec not in rows: sys.exit(f"{ref} not in toc")
    d, st, en, title = rows[sec]
    print(f"{BOOKS[b][0]} {sec} {title}  pdf pages {st}-{en}")
    for s, (d2, st2, en2, t2) in rows.items():
        if s.startswith(sec + "."): print(f"  {'  ' * (d2 - d - 1)}{s} {t2}  p{st2}-{en2}")
    print(f'pdftotext -layout -f {st} -l {en} "{os.path.join(BOOK_DIR, BOOKS[b][1])}" -')

def read(ref, toc):
    b, _, sec = ref.partition(":")
    d, st, en, title = toc[b][sec]
    out = subprocess.run(["pdftotext", "-layout", "-f", str(st), "-l", str(en),
                          os.path.join(BOOK_DIR, BOOKS[b][1]), "-"], capture_output=True)
    print(out.stdout.decode("utf-8", "replace"))

def check(toc, mat):
    bad = 0
    for r in mat:
        for ref in r["refs"]:
            b, _, sec = ref.partition(":")
            if sec not in toc.get(b, {}): print("missing ref", r["id"], ref); bad += 1
        for p in r["code"]:
            if not os.path.exists(code_path(p)): print("missing code", r["id"], p); bad += 1
    ids = [r["id"] for r in mat]
    if len(ids) != len(set(ids)): print("duplicate ids"); bad += 1
    print(f"{len(mat)} rows, {sum(len(r['refs']) for r in mat)} refs, {bad} problems")
    return bad

def main(a):
    toc, mat = load_toc(), load_matrix()
    if not a: print(__doc__); return
    as_json = "--json" in a
    if as_json:
        a = [x for x in a if x != "--json"]
    if not a:
        if as_json:
            sys.exit("--json needs search terms, --code, --niche, or --row")
        print(__doc__); return
    if a[0] == "--code":
        if len(a) < 2: sys.exit("--code needs a path")
        by_code(a[1], toc, mat, as_json=as_json)
    elif a[0] == "--row":
        if len(a) < 2: sys.exit("--row needs an id")
        by_row(a[1], toc, mat, as_json=as_json)
    elif a[0] == "--brief":
        if len(a) < 2: sys.exit("--brief needs a code path")
        by_brief(a[1], toc, mat)
    elif a[0] == "--sec": section(a[1], toc)
    elif a[0] == "--read": read(a[1], toc)
    elif a[0] == "--niche":
        domain = a[1] if len(a) >= 2 else None
        by_niche(toc, mat, domain, as_json=as_json)
    elif a[0] == "--check": sys.exit(1 if check(toc, mat) else 0)
    else: search(a, toc, mat, as_json=as_json)

if __name__ == "__main__":
    main(sys.argv[1:])
