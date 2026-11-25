"""Rebuild toc/<book>.tsv from the PDFs (deterministic; no book text is stored).

    python -I src/dev/ref_books_index/build_toc.py [<books dir>]   # default: $REF_BOOKS_DIR

Sources per book: PDF bookmark outline (pypdf) + numbered x.y.z[.w] headings found in the
pdftotext dump (GEA books only list x.y in their outline). Output columns:

    sec  depth  start  end  title

`start`/`end` are **PDF page indices** (1-based, what `pdftotext -f/-l` takes), not printed
page numbers. `end` = start page of the next entry at the same or shallower depth.
"""
import os, re, subprocess, sys, tempfile
import pypdf

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = sys.argv[1] if len(sys.argv) > 1 else os.environ.get("REF_BOOKS_DIR", "")
if not ROOT:
    sys.exit("build_toc: pass the books folder or set REF_BOOKS_DIR")
BOOKS = {  # id -> (file, scan_text_for_subsections, first_body_page)
    "gea4v1": ("game-engine-architecture-volume-i-foundations-and-core-engine-systems-4_compress.pdf", True, 15),
    "gea4v2": ("Game Engine Architecture Volume II_ Graphics_ Motion_ and Sound_ 4th Edition.sanet.st.pdf", True, 15),
    "gea3":   ("Gregory, Jason - Game engine architecture.pdf", True, 18),
    "aig3":   ("ai-for-games-3e_compress.pdf", False, 20),
}
SKIP = re.compile(r"^(cover|half title|title page|copyright|dedication|contents|table of contents|preface)", re.I)
ROMAN = re.compile(r"^(?:PART\s+)?([IVX]+)[:.]?\s+(.*)$")

def parse_title(t):
    t = "".join(c if c.isprintable() else " " for c in t)
    t = re.sub(r"\s+", " ", t).strip()
    m = re.match(r"^CHAPTER\s+(\d+):?\s*(.*)$", t, re.I)
    if m: return m.group(1), m.group(2).title()
    m = re.match(r"^(\d+(?:\.\d+)*)\.?\s+(.*)$", t)
    if m: return m.group(1), m.group(2)
    m = ROMAN.match(t)
    if m and t.split()[0].rstrip(":").upper() in ("I","II","III","IV","V","PART"): return "P" + m.group(1), m.group(2).title()
    return None, t

def outline(reader):
    rows = []
    def walk(items):
        for it in items:
            if isinstance(it, list): walk(it); continue
            try: pg = reader.get_destination_page_number(it) + 1
            except Exception: continue
            rows.append((pg, it.title))
    walk(reader.outline)
    return rows

HEAD = re.compile(r"^(\d{1,2}(?:\.\d{1,2}){2,3})\.?\s+([A-Z][^\n]{2,90})$")
HEAD_NUM = re.compile(r"^(\d{1,2}(?:\.\d{1,2}){2,3})\.?$")

def text_subsections(pdf, first_page):
    with tempfile.TemporaryDirectory() as td:
        out = os.path.join(td, "b.txt")
        subprocess.run(["pdftotext", pdf, out], check=True, capture_output=True)
        pages = open(out, encoding="utf-8", errors="replace").read().split("\f")
    found = {}
    for pi, p in enumerate(pages):
        if pi + 1 < first_page: continue
        lines = [l.strip() for l in p.split("\n")]
        for i, l in enumerate(lines):
            m = HEAD.match(l); sec = title = None
            if m: sec, title = m.group(1), m.group(2)
            else:
                m = HEAD_NUM.match(l)
                if m and i + 1 < len(lines) and re.match(r"^[A-Z][^\n]{2,90}$", lines[i + 1]):
                    sec, title = m.group(1), lines[i + 1]
            if not sec or sec in found: continue
            if re.search(r"\.\s*\d+$|\.{3,}|,$", title) or len(title.split()) > 12: continue
            found[sec] = (pi + 1, title)
    return found

def key(sec):
    if sec.startswith("P"): return (0,)
    return tuple(int(x) for x in sec.split("."))

def build(bid, fname, scan, first):
    pdf = os.path.join(ROOT, fname)
    r = pypdf.PdfReader(pdf)
    rows = {}
    for pg, t in outline(r):
        if SKIP.match(t.strip()): continue
        sec, title = parse_title(t)
        if sec is None:
            sec = "X" + re.sub(r"\W+", "_", title.lower())[:24]
        rows.setdefault(sec, (pg, title))
    if scan:
        for sec, v in text_subsections(pdf, first).items():
            ch = sec.split(".")[0]
            # keep only if its chapter exists and page is not before its parent
            parent = ".".join(sec.split(".")[:-1])
            if ch in rows and (parent not in rows or rows[parent][0] <= v[0]):
                rows.setdefault(sec, v)
    def depth(s):
        if s.startswith("P"): return 0
        if s.startswith("X"): return 1
        return s.count(".") + 1
    items = sorted(rows.items(), key=lambda kv: (kv[1][0], depth(kv[0]), key(kv[0]) if kv[0][0].isdigit() else (0,)))
    last = len(r.pages)
    out = []
    for i, (sec, (pg, title)) in enumerate(items):
        d = depth(sec); end = last
        for sec2, (pg2, _) in items[i + 1:]:
            if depth(sec2) <= d: end = max(pg, pg2); break
        out.append(f"{sec}\t{d}\t{pg}\t{end}\t{title}")
    path = os.path.join(HERE, "toc", f"{bid}.tsv")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write("sec\tdepth\tstart\tend\ttitle\n" + "\n".join(out) + "\n")
    print(bid, len(out), "rows", last, "pages")

if __name__ == "__main__":
    for bid, (fname, scan, first) in BOOKS.items():
        build(bid, fname, scan, first)
