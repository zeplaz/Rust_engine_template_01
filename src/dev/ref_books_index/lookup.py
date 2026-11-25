"""Cheap ref-book lookup for agents. Stdlib only; reads matrix.md + toc/*.tsv (never the PDFs,
except `--read`, which shells out to pdftotext for just the requested pages).

    python src/dev/ref_books_index/lookup.py fire spread            # concept -> book pages + code
    python src/dev/ref_books_index/lookup.py --code combat/relax.rs  # file -> which book sections apply
    python src/dev/ref_books_index/lookup.py --sec gea4v2:17.6.3     # section subtree + page range
    python src/dev/ref_books_index/lookup.py --read aig3:5.7.7       # print that section's text
    python src/dev/ref_books_index/lookup.py --niche [DOMAIN]        # list ★ rows
    python src/dev/ref_books_index/lookup.py --open [DOMAIN]         # ★ rows not ADOPTED/REJECTED
    python src/dev/ref_books_index/lookup.py --check                 # validate refs + code paths + adoption
    python src/dev/ref_books_index/lookup.py --json fire spread      # same as search, JSON array
    python src/dev/ref_books_index/lookup.py --json --code PATH      # --code / --niche also take --json
    python src/dev/ref_books_index/lookup.py --row PHYS-02           # one matrix row + refs
    python src/dev/ref_books_index/lookup.py --brief combat/ballistics.rs  # ≤12 lines: id, best ref, --read
    python src/dev/ref_books_index/lookup.py --grep REGEX book:sec   # match lines + context + page
    python src/dev/ref_books_index/lookup.py --read aig3:6.2.8 --pages  # page range + line count only
    python src/dev/ref_books_index/lookup.py --gaps                  # uncovered src+tools; [tools]/[voxel]/[atmos] tags
    python src/dev/ref_books_index/lookup.py --dual shape grammar    # NEW matrix + LEGACY keywords (both)
    python src/dev/ref_books_index/lookup.py --dual --code PATH      # same for a code path
    python src/dev/ref_books_index/lookup.py --json --dual shape grammar  # {"new":[...],"legacy":[...]}
"""
import csv, json, os, re, shutil, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
# Books live outside the repo: set REF_BOOKS_DIR (and optionally REF_BOOKS_LEGACY_LOOKUP_DIR).
BOOK_DIR = os.environ.get("REF_BOOKS_DIR", "")
LEGACY_LOOKUP_DIR = os.environ.get(
    "REF_BOOKS_LEGACY_LOOKUP_DIR", os.path.join(BOOK_DIR, "lookup") if BOOK_DIR else ""
)
CACHE_DIR = os.path.join(REPO, "debug_runs", "ref_books_cache")
ADOPTION_PATH = os.path.join(HERE, "adoption.tsv")
# UNWIRED = code exists but no production caller (review 2026-11-16); ADOPTED requires a prod caller.
STATUSES = {"OPEN", "ADOPTED", "PARTIAL", "UNWIRED", "DEFER", "REJECTED"}
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

def load_adoption():
    """id -> {status, evidence, date}. Missing file → empty."""
    out = {}
    if not os.path.exists(ADOPTION_PATH):
        return out
    with open(ADOPTION_PATH, encoding="utf-8") as f:
        header = f.readline()
        for line in f:
            line = line.rstrip("\n")
            if not line: continue
            parts = line.split("\t")
            while len(parts) < 4:
                parts.append("")
            rid, status, evidence, date = parts[0], parts[1], parts[2], parts[3]
            out[rid] = {"status": status, "evidence": evidence, "date": date}
    return out

def status_tag(adopt, rid):
    a = adopt.get(rid)
    if not a: return ""
    return f"[{a['status']}]"

def id_display(adopt, rid):
    tag = status_tag(adopt, rid)
    return f"{rid} {tag}".rstrip() if tag else rid

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

def row_json(toc, r, adopt):
    o = {
        "id": r["id"],
        "tier": r["tier"],
        "concept": r["concept"],
        "refs": [ref_obj(toc, ref) for ref in r["refs"]],
        "code": list(r["code"]),
        "take": r["take"],
    }
    a = adopt.get(r["id"])
    if a:
        o["status"] = a["status"]
    return o

def emit_json(rows, toc, adopt):
    print(json.dumps([row_json(toc, r, adopt) for r in rows], ensure_ascii=False))

def show_row(toc, r, adopt, refs=True):
    print(f"{id_display(adopt, r['id'])} {r['tier']} {r['concept']}")
    if refs:
        for ref in r["refs"]: print(ref_line(toc, ref))
    if r["code"]: print("    code: " + "; ".join(r["code"]))
    print("    -> " + r["take"])

def load_legacy_lookup():
    """Rows from keyword_lookup.csv, or None if legacy tree absent."""
    path = os.path.join(LEGACY_LOOKUP_DIR, "keyword_lookup.csv")
    if not os.path.isfile(path):
        return None
    rows = []
    with open(path, encoding="utf-8", newline="") as f:
        for row in csv.DictReader(f):
            page = row.get("pdf_page", "").strip()
            if not page or not row.get("keyword", "").strip():
                continue
            rows.append({
                "keyword": row["keyword"],
                "book_id": row["book_id"],
                "section": row.get("section", ""),
                "pdf_page": int(page),
            })
    return rows

def score_legacy(terms, legacy_rows):
    terms = [t.lower() for t in terms]
    scored = []
    for row in legacy_rows:
        kw = row["keyword"].replace("_", " ").lower()
        sec = row["section"].lower()
        hay = kw + " " + sec
        if not all(t in hay for t in terms):
            continue
        s = sum(3 * (t in kw) + (t in sec) for t in terms)
        scored.append((s, row))
    scored.sort(key=lambda x: (-x[0], x[1]["keyword"], x[1]["book_id"]))
    return scored

def legacy_line(row):
    return f"    {row['keyword']:<22} {row['book_id']:<10} p{row['pdf_page']:<5} {row['section']}"

def terms_from_code(frag, mat_rows):
    frag = frag.replace("\\", "/").removeprefix("src/")
    terms = []
    seen = set()
    for part in re.split(r"[/_.\\-]+", frag):
        w = part.lower()
        if len(w) >= 3 and w not in seen:
            seen.add(w)
            terms.append(w)
    for r in mat_rows:
        for blob in (r["kw"], r["concept"]):
            for word in re.split(r"[,;/]+", blob):
                w = word.strip().lower()
                if len(w) >= 3 and w not in seen:
                    seen.add(w)
                    terms.append(w)
    return terms

def by_dual(terms, toc, mat, adopt, as_json=False):
    scored = score_search(terms, mat)
    new_hits = [r for _, r in scored[:8]]
    legacy_rows = load_legacy_lookup()
    legacy_hits = []
    if legacy_rows is not None:
        legacy_hits = [row for _, row in score_legacy(terms, legacy_rows)[:12]]
    if as_json:
        out = {"new": [row_json(toc, r, adopt) for r in new_hits], "legacy": legacy_hits}
        print(json.dumps(out, ensure_ascii=False))
        return
    print(f"== dual: new matrix ({len(scored)} rows) ==")
    for _, r in scored[:8]:
        show_row(toc, r, adopt)
    if legacy_rows is None:
        print("legacy: unavailable")
        return
    print(f"== dual: legacy keywords ({len(legacy_hits)} rows) ==")
    if not legacy_hits:
        print("    (no keyword hits)")
    else:
        for row in legacy_hits:
            print(legacy_line(row))

def by_dual_code(frag, toc, mat, adopt, as_json=False):
    rows = rows_for_code(frag, mat)
    legacy_rows = load_legacy_lookup()
    terms = terms_from_code(frag, rows)
    legacy_hits = []
    if legacy_rows is not None and terms:
        legacy_hits = [row for _, row in score_legacy(terms, legacy_rows)[:12]]
    if as_json:
        out = {"new": [row_json(toc, r, adopt) for r in rows], "legacy": legacy_hits}
        print(json.dumps(out, ensure_ascii=False))
        return
    print(f"== dual: new matrix ({len(rows)} rows) ==")
    for r in rows:
        show_row(toc, r, adopt)
    if legacy_rows is None:
        print("legacy: unavailable")
        return
    print(f"== dual: legacy keywords ({len(legacy_hits)} rows) ==")
    if not legacy_hits:
        print("    (no keyword hits)")
    else:
        for row in legacy_hits:
            print(legacy_line(row))

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

def search(terms, toc, mat, adopt, as_json=False):
    scored = score_search(terms, mat)
    if as_json:
        emit_json([r for _, r in scored[:8]], toc, adopt)
        return
    print(f"== matrix ({len(scored)} rows) ==")
    for _, r in scored[:8]: show_row(toc, r, adopt)
    hits = []
    for b, rows in toc.items():
        for sec, (d, st, en, title) in rows.items():
            t = title.lower()
            if all(x in t for x in terms): hits.append((b, sec, st, en, title))
    if any(h[0] != "gea3" for h in hits):
        hits = [h for h in hits if h[0] != "gea3"]
    print(f"== toc titles ({len(hits)}) ==")
    for b, sec, st, en, title in hits[:20]: print(f"    {b:<6} {sec:<10} p{st}-{en:<5} {title}")

def rows_for_code(frag, mat):
    frag = frag.replace("\\", "/").removeprefix("src/")
    return [r for r in mat if any(frag in p or p.rstrip("/") in frag for p in r["code"])]

def by_code(frag, toc, mat, adopt, as_json=False):
    frag = frag.replace("\\", "/").removeprefix("src/")
    rows = rows_for_code(frag, mat)
    if as_json:
        emit_json(rows, toc, adopt)
        return
    print(f"== rows touching {frag} ({len(rows)}) ==")
    for r in rows: show_row(toc, r, adopt)

def niche_rows(mat, domain=None):
    out = []
    for r in mat:
        if r["tier"] == "★" and (domain is None or r["id"].startswith(domain.upper())):
            out.append(r)
    return out

def by_niche(toc, mat, adopt, domain=None, as_json=False):
    rows = niche_rows(mat, domain)
    if as_json:
        emit_json(rows, toc, adopt)
        return
    for r in rows:
        show_row(toc, r, adopt, refs=False)

def by_open(toc, mat, adopt, domain=None, as_json=False):
    rows = []
    for r in niche_rows(mat, domain):
        st = adopt.get(r["id"], {}).get("status")
        if st in ("ADOPTED", "REJECTED"):
            continue
        rows.append(r)
    if as_json:
        emit_json(rows, toc, adopt)
        return
    for r in rows:
        show_row(toc, r, adopt, refs=False)

def best_ref(toc, r):
    for ref in r["refs"]:
        b, _, sec = ref.partition(":")
        if toc.get(b, {}).get(sec):
            return ref
    return r["refs"][0] if r["refs"] else None

def by_row(row_id, toc, mat, adopt, as_json=False):
    want = row_id.upper()
    rows = [r for r in mat if r["id"] == want]
    if not rows:
        sys.exit(f"unknown row {row_id}")
    if as_json:
        emit_json(rows, toc, adopt)
        return
    show_row(toc, rows[0], adopt, refs=True)

def by_brief(frag, toc, mat, adopt):
    rows = rows_for_code(frag, mat)
    lines = 0
    for r in rows:
        if lines >= 12:
            break
        ref = best_ref(toc, r)
        tag = status_tag(adopt, r["id"])
        prefix = f"{r['id']} {tag}".rstrip() if tag else r["id"]
        if not ref:
            print(f"{prefix}  (no refs)")
            lines += 1
            continue
        print(f"{prefix}  {ref}  →  python src/dev/ref_books_index/lookup.py --read {ref}")
        lines += 1

def cache_path(book, sec):
    safe = sec.replace(".", "_")
    return os.path.join(CACHE_DIR, f"{book}_{safe}.txt")

def fetch_section_text(ref, toc, use_cache=True):
    b, _, sec = ref.partition(":")
    if sec not in toc.get(b, {}):
        sys.exit(f"{ref} not in toc")
    d, st, en, title = toc[b][sec]
    path = cache_path(b, sec)
    if use_cache and os.path.exists(path):
        with open(path, encoding="utf-8") as f:
            text = f.read()
        print("cached", file=sys.stderr)
        return text, st, en, title, True
    exe = shutil.which("pdftotext")
    if not exe:
        sys.exit(
            "pdftotext not on PATH (Poppler). Install it, or pre-seed "
            f"{path} so --read can use the cache."
        )
    out = subprocess.run(
        [exe, "-layout", "-f", str(st), "-l", str(en),
         os.path.join(BOOK_DIR, BOOKS[b][1]), "-"],
        capture_output=True,
    )
    text = out.stdout.decode("utf-8", "replace")
    os.makedirs(CACHE_DIR, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)
    return text, st, en, title, False

def section(ref, toc):
    b, _, sec = ref.partition(":")
    rows = toc[b]
    if sec not in rows: sys.exit(f"{ref} not in toc")
    d, st, en, title = rows[sec]
    print(f"{BOOKS[b][0]} {sec} {title}  pdf pages {st}-{en}")
    for s, (d2, st2, en2, t2) in rows.items():
        if s.startswith(sec + "."): print(f"  {'  ' * (d2 - d - 1)}{s} {t2}  p{st2}-{en2}")
    print(f'pdftotext -layout -f {st} -l {en} "{os.path.join(BOOK_DIR, BOOKS[b][1])}" -')

def read(ref, toc, pages_only=False):
    text, st, en, title, _ = fetch_section_text(ref, toc, use_cache=True)
    if pages_only:
        nlines = text.count("\n") + (0 if text.endswith("\n") or not text else 1)
        print(f"{ref}  pdf pages {st}-{en}  lines {nlines}  {title}")
        return
    print(text, end="" if text.endswith("\n") else "\n")

def grep_section(pattern, ref, toc):
    try:
        rx = re.compile(pattern)
    except re.error as e:
        sys.exit(f"bad regex: {e}")
    text, st, en, title, _ = fetch_section_text(ref, toc, use_cache=True)
    lines = text.splitlines()
    # Approximate page from line index across [st, en]
    span = max(en - st + 1, 1)
    shown = 0
    for i, line in enumerate(lines):
        if not rx.search(line):
            continue
        # estimate PDF page
        page = st + min(span - 1, (i * span) // max(len(lines), 1))
        lo = max(0, i - 2)
        hi = min(len(lines), i + 3)
        print(f"-- {ref} ~p{page} line {i+1} --")
        for j in range(lo, hi):
            mark = ">" if j == i else " "
            print(f"{mark} {lines[j]}")
        print()
        shown += 1
        if shown >= 40:
            print("(truncated)")
            break
    if shown == 0:
        print(f"no matches for /{pattern}/ in {ref}")

def covered_code_paths(mat):
    paths = set()
    for r in mat:
        for p in r["code"]:
            p = p.replace("\\", "/")
            if p.startswith("tools/"):
                paths.add(p.rstrip("/"))
            else:
                paths.add(p.removeprefix("src/").rstrip("/"))
    return paths

def _gap_path_covered(rel, covered, *, dir_entry=False):
    rel = rel.replace("\\", "/").rstrip("/")
    bodies = {rel}
    if rel.startswith("src/"):
        bodies.add(rel[4:])
    elif rel.startswith("tools/"):
        bodies.add(rel[6:])
    for body in bodies:
        top = body.split("/")[0] if body else ""
        for c in covered:
            c = c.replace("\\", "/").rstrip("/")
            if not c:
                continue
            if body == c or body.startswith(c + "/") or c.startswith(body + "/"):
                return True
            if dir_entry and top and (c == top or c.startswith(top + "/")):
                return True
            cr = c.rstrip("/")
            if cr in body or body.startswith(cr + "/") or body.startswith(cr):
                return True
    return False

def _count_source_lines(root_dir, suffixes):
    n = 0
    for root, _, files in os.walk(root_dir):
        for fn in files:
            if not any(fn.endswith(s) for s in suffixes):
                continue
            try:
                with open(os.path.join(root, fn), encoding="utf-8", errors="replace") as f:
                    n += sum(1 for _ in f)
            except OSError:
                pass
    return n

def gap_bucket_tags(rel_path):
    """PLAN v3 T2: tools / voxel / atmos buckets for --gaps lines."""
    p = rel_path.replace("\\", "/").lower()
    tags = []
    if p.startswith("tools/") or "/tools/" in p:
        tags.append("tools")
    if "voxel" in p or "massing" in p:
        tags.append("voxel")
    if "atmosphere" in p or "atmos" in p:
        tags.append("atmos")
    return tags

def _collect_tree_gaps(tree_root, rel_prefix, covered, *, dir_suffixes, large_suffixes, large_min):
    dir_hits, file_hits = [], []
    if not os.path.isdir(tree_root):
        return dir_hits, file_hits
    for name in sorted(os.listdir(tree_root)):
        d = os.path.join(tree_root, name)
        if not os.path.isdir(d):
            continue
        rel = f"{rel_prefix}{name}/"
        if _gap_path_covered(rel, covered, dir_entry=True) or _gap_path_covered(
            name + "/", covered, dir_entry=True
        ):
            continue
        n = _count_source_lines(d, dir_suffixes)
        dir_hits.append((n, rel))
    for root, _, files in os.walk(tree_root):
        for fn in files:
            if not any(fn.endswith(s) for s in large_suffixes):
                continue
            full = os.path.join(root, fn)
            try:
                with open(full, encoding="utf-8", errors="replace") as f:
                    n = sum(1 for _ in f)
            except OSError:
                continue
            if n <= large_min:
                continue
            rel = os.path.relpath(full, tree_root).replace("\\", "/")
            rel_path = f"{rel_prefix}{rel}"
            if _gap_path_covered(rel_path, covered, dir_entry=False):
                continue
            file_hits.append((n, rel_path))
    return dir_hits, file_hits

def by_gaps(mat):
    covered = covered_code_paths(mat)
    src_root = os.path.join(REPO, "src")
    tools_root = os.path.join(REPO, "tools")
    dir_hits, file_hits = _collect_tree_gaps(
        src_root, "src/", covered, dir_suffixes=(".rs",), large_suffixes=(".rs",), large_min=400,
    )
    t_dirs, t_files = _collect_tree_gaps(
        tools_root, "tools/", covered, dir_suffixes=(".py", ".rs"), large_suffixes=(".py", ".rs"), large_min=400,
    )
    dir_hits.extend(t_dirs)
    file_hits.extend(t_files)
    def _gap_rank(item):
        n, p = item
        return (0 if gap_bucket_tags(p) else 1, -n, p)

    all_hits = sorted(dir_hits + file_hits, key=_gap_rank)
    ranked = all_hits[:40]
    print(f"== gaps (top {len(ranked)}) ==")
    for n, p in ranked:
        tags = gap_bucket_tags(p)
        tag = f"[{','.join(tags)}] " if tags else ""
        print(f"{n:6d}  {tag}{p}")
    tagged = [(n, p, gap_bucket_tags(p)) for n, p in all_hits if gap_bucket_tags(p)][:25]
    if tagged:
        print(f"== gap buckets ({len(tagged)} tagged paths) ==")
        for n, p, tags in tagged:
            print(f"    {','.join(tags):<12} {n:6d}  {p}")

def check(toc, mat, adopt):
    bad = 0
    mat_ids = {r["id"] for r in mat}
    for r in mat:
        for ref in r["refs"]:
            b, _, sec = ref.partition(":")
            if sec not in toc.get(b, {}): print("missing ref", r["id"], ref); bad += 1
        for p in r["code"]:
            if not os.path.exists(code_path(p)): print("missing code", r["id"], p); bad += 1
    ids = [r["id"] for r in mat]
    if len(ids) != len(set(ids)): print("duplicate ids"); bad += 1
    for rid, a in adopt.items():
        if rid not in mat_ids:
            print("adoption unknown id", rid); bad += 1
        if a["status"] not in STATUSES:
            print("adoption bad status", rid, a["status"]); bad += 1
    print(f"{len(mat)} rows, {sum(len(r['refs']) for r in mat)} refs, {bad} problems")
    return bad

def main(a):
    toc, mat, adopt = load_toc(), load_matrix(), load_adoption()
    if not a: print(__doc__); return
    as_json = "--json" in a
    pages_only = "--pages" in a
    if as_json:
        a = [x for x in a if x != "--json"]
    if pages_only:
        a = [x for x in a if x != "--pages"]
    if not a:
        if as_json:
            sys.exit("--json needs search terms, --code, --niche, --open, or --row")
        print(__doc__); return
    if a[0] == "--code":
        if len(a) < 2: sys.exit("--code needs a path")
        by_code(a[1], toc, mat, adopt, as_json=as_json)
    elif a[0] == "--row":
        if len(a) < 2: sys.exit("--row needs an id")
        by_row(a[1], toc, mat, adopt, as_json=as_json)
    elif a[0] == "--brief":
        if len(a) < 2: sys.exit("--brief needs a code path")
        by_brief(a[1], toc, mat, adopt)
    elif a[0] == "--sec": section(a[1], toc)
    elif a[0] == "--read":
        if len(a) < 2: sys.exit("--read needs book:sec")
        read(a[1], toc, pages_only=pages_only)
    elif a[0] == "--grep":
        if len(a) < 3: sys.exit("--grep needs <regex> <book:sec>")
        grep_section(a[1], a[2], toc)
    elif a[0] == "--niche":
        domain = a[1] if len(a) >= 2 else None
        by_niche(toc, mat, adopt, domain, as_json=as_json)
    elif a[0] == "--open":
        domain = a[1] if len(a) >= 2 else None
        by_open(toc, mat, adopt, domain, as_json=as_json)
    elif a[0] == "--gaps":
        by_gaps(mat)
    elif a[0] == "--dual":
        if len(a) < 2:
            sys.exit("--dual needs search terms or --code PATH")
        if a[1] == "--code":
            if len(a) < 3:
                sys.exit("--dual --code needs a path")
            by_dual_code(a[2], toc, mat, adopt, as_json=as_json)
        else:
            by_dual(a[1:], toc, mat, adopt, as_json=as_json)
    elif a[0] == "--check": sys.exit(1 if check(toc, mat, adopt) else 0)
    else: search(a, toc, mat, adopt, as_json=as_json)

if __name__ == "__main__":
    main(sys.argv[1:])
