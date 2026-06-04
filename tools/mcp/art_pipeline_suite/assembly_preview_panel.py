"""APS-PREVIEW-002 — assembly snapshot preview panel (three.js / optional Bevy thumb)."""

from __future__ import annotations

import tkinter as tk
from pathlib import Path
from tkinter import messagebox, ttk

try:
    from PIL import Image, ImageTk
except ImportError:  # pragma: no cover
    Image = None  # type: ignore[misc, assignment]
    ImageTk = None  # type: ignore[misc, assignment]

from rust_engine_mcp import assembly_preview
from rust_engine_mcp.paths import repo_root


class AssemblyPreviewPanel(ttk.LabelFrame):
    def __init__(self, master: tk.Misc, *, on_log=None) -> None:
        super().__init__(master, text="3D preview", padding=6)
        self._on_log = on_log or (lambda _line: None)
        self._snapshot: dict | None = None
        self._thumb_photo: ImageTk.PhotoImage | None = None
        self._last_result: dict | None = None
        self._build()

    def _build(self) -> None:
        ttk.Label(
            self,
            text="Bevy worker when built; else browser three.js. RUST_ENGINE_BEVY_PREVIEW=0 forces browser.",
            wraplength=360,
            justify=tk.LEFT,
            font=("Segoe UI", 8),
            foreground="#555",
        ).pack(anchor=tk.W, pady=(0, 4))

        body = ttk.Panedwindow(self, orient=tk.HORIZONTAL)
        body.pack(fill=tk.BOTH, expand=True)

        thumb_pane = ttk.Frame(body, width=200)
        body.add(thumb_pane, weight=0)
        self._thumb_label = ttk.Label(
            thumb_pane,
            text="(Preview assembly)",
            anchor=tk.CENTER,
            width=22,
        )
        self._thumb_label.pack(fill=tk.BOTH, expand=True, padx=4, pady=4)

        controls = ttk.Frame(body, padding=(4, 0))
        body.add(controls, weight=1)

        btn_row = ttk.Frame(controls)
        btn_row.pack(anchor=tk.W, pady=2)
        ttk.Button(btn_row, text="Preview assembly", command=self.on_preview).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_row, text="Open URL", command=self.on_open_url).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_row, text="Copy URL", command=self.on_copy_url).pack(side=tk.LEFT, padx=2)

        self._status_var = tk.StringVar(value="")
        ttk.Label(controls, textvariable=self._status_var, wraplength=320, justify=tk.LEFT).pack(
            anchor=tk.W, pady=4
        )
        self._url_var = tk.StringVar(value="")
        url_row = ttk.Frame(controls)
        url_row.pack(anchor=tk.W, fill=tk.X)
        ttk.Label(url_row, text="URL:").pack(side=tk.LEFT)
        self._url_entry = ttk.Entry(url_row, textvariable=self._url_var, state="readonly")
        self._url_entry.pack(side=tk.LEFT, padx=4, fill=tk.X, expand=True)

        def _sync_panes(_event=None) -> None:
            try:
                body.paneconfigure(thumb_pane, width=max(160, min(240, body.winfo_width() // 3)))
            except tk.TclError:
                pass

        body.bind("<Configure>", _sync_panes)

    def set_snapshot(self, snapshot: dict | None) -> None:
        self._snapshot = snapshot
        aid = (snapshot or {}).get("assembly_id")
        if aid:
            self._status_var.set(f"Snapshot {aid} — click Preview assembly")

    def on_preview(self) -> None:
        if not self._snapshot:
            messagebox.showinfo("Preview", "Generate or load an assembly snapshot first.")
            self._on_log("preview skipped — no snapshot loaded")
            return
        aid = self._snapshot.get("assembly_id", "?")
        placements = len(self._snapshot.get("module_placements") or [])
        self._on_log(f"preview start {aid} · {placements} placements")
        try:
            result = assembly_preview.preview_assembly_from_dict(self._snapshot, open_browser=True)
        except Exception as exc:  # noqa: BLE001
            self._on_log(f"preview error: {exc}")
            messagebox.showerror("Preview", str(exc))
            return
        self._last_result = result
        url = str(result.get("preview_url") or "")
        self._url_var.set(url)
        assembly_preview.write_aps_preview_002_witness(result)
        mode = result.get("mode", "?")
        loaded = result.get("modules_loaded", 0)
        missing = result.get("missing_glb") or []
        profiles = ", ".join(result.get("material_profiles_sample") or []) or "—"
        status = f"{mode} · {loaded} modules · profiles: {profiles}"
        if missing:
            status += f" · missing GLB: {len(missing)}"
        if url:
            status += " · browser opened (or use Open last URL)"
        else:
            status += " · no URL (Bevy mode or zero placements)"
        self._status_var.set(status)
        self._on_log(f"preview done {result.get('assembly_id')} · {mode} · url={url or '—'}")
        self._load_thumbnail(result.get("png") or "")
        if url:
            messagebox.showinfo(
                "Preview",
                f"Three.js preview at:\n{url}\n\nIf the browser did not open, use Open last URL or Copy URL.",
            )

    def on_open_url(self) -> None:
        url = self._url_var.get().strip()
        if not url.startswith("http"):
            messagebox.showinfo(
                "Preview",
                "Run Preview assembly first.\n\nRequires a loaded snapshot with resolved GLB paths.",
            )
            self._on_log("open url skipped — no preview_url")
            return
        import webbrowser

        self._on_log(f"open url {url}")
        webbrowser.open(url)

    def on_copy_url(self) -> None:
        url = self._url_var.get().strip()
        if not url.startswith("http"):
            messagebox.showinfo("Preview", "No URL yet — run Preview assembly first.")
            return
        self.clipboard_clear()
        self.clipboard_append(url)
        self._on_log(f"copied url {url}")
        self._status_var.set(f"Copied: {url}")

    def _load_thumbnail(self, png_rel: str) -> None:
        if not png_rel or Image is None or ImageTk is None:
            return
        path = repo_root() / str(png_rel).replace("\\", "/")
        if not path.is_file():
            self._on_log(f"preview thumb missing {path.name}")
            return
        img = Image.open(path).convert("RGB")
        img.thumbnail((200, 200), Image.Resampling.LANCZOS)
        self._thumb_photo = ImageTk.PhotoImage(img)
        self._thumb_label.configure(image=self._thumb_photo, text="")
