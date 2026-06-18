"""APS-PREVIEW-002 — assembly snapshot preview panel (three.js / optional Bevy thumb)."""

from __future__ import annotations

import tkinter as tk
from tkinter import ttk

try:
    from PIL import Image, ImageTk
except ImportError:  # pragma: no cover
    Image = None  # type: ignore[misc, assignment]
    ImageTk = None  # type: ignore[misc, assignment]

from rust_engine_mcp import assembly_preview
from rust_engine_mcp.paths import repo_root

from .aps_inline_feedback import apply_status_atom, set_inline_status
from .aps_preview_state import apply_preview_photo, configure_preview_label, image_is_near_black, make_fidelity_chip
from .aps_theme import COLOR_MUTED, FONT_SMALL, PREVIEW_THUMB_MD
from .job_controller import JobRecord, JobResult


class AssemblyPreviewPanel(ttk.LabelFrame):
    def __init__(
        self,
        master: tk.Misc,
        *,
        on_log=None,
        on_preview_thumb=None,
        start_job=None,
    ) -> None:
        super().__init__(master, text="Assembly preview", padding=6)
        self._on_log = on_log or (lambda _line: None)
        self._on_preview_thumb = on_preview_thumb
        self._start_job = start_job
        self._snapshot: dict | None = None
        self._thumb_photo: ImageTk.PhotoImage | None = None
        self._last_result: dict | None = None
        self._preview_btn: ttk.Button | None = None
        self._build()

    def _build(self) -> None:
        ttk.Label(
            self,
            text="Whole assembly in 3D (browser or built-in viewer). Generate or load a snapshot first.",
            wraplength=360,
            justify=tk.LEFT,
            font=FONT_SMALL,
            foreground=COLOR_MUTED,
        ).pack(anchor=tk.W, pady=(0, 4))

        body = ttk.Panedwindow(self, orient=tk.HORIZONTAL)
        body.pack(fill=tk.BOTH, expand=True)

        thumb_pane = ttk.Frame(body, width=200)
        body.add(thumb_pane, weight=0)
        make_fidelity_chip(thumb_pane, "interactive").pack(anchor=tk.W, padx=4, pady=(4, 0))
        self._thumb_label = tk.Label(thumb_pane, relief=tk.SUNKEN)
        self._thumb_label.pack(fill=tk.BOTH, expand=True, padx=4, pady=4)
        configure_preview_label(
            self._thumb_label,
            "empty",
            detail="No Assembly loaded — generate or load one first",
            width=PREVIEW_THUMB_MD,
            height=PREVIEW_THUMB_MD,
        )

        controls = ttk.Frame(body, padding=(4, 0))
        body.add(controls, weight=1)

        btn_row = ttk.Frame(controls)
        btn_row.pack(anchor=tk.W, pady=2)
        self._preview_btn = ttk.Button(btn_row, text="Preview assembly", command=self.on_preview)
        self._preview_btn.pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_row, text="Open URL", command=self.on_open_url).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_row, text="Copy URL", command=self.on_copy_url).pack(side=tk.LEFT, padx=2)

        self._status_var = tk.StringVar(value="")
        self._status_lbl = ttk.Label(controls, textvariable=self._status_var, wraplength=320, justify=tk.LEFT)
        self._status_lbl.pack(anchor=tk.W, pady=4)
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

    def _set_status(self, text: str, *, ok: bool | None = None) -> None:
        set_inline_status(self._status_lbl, self._status_var, text, ok=ok)

    def set_snapshot(self, snapshot: dict | None) -> None:
        self._snapshot = snapshot
        if snapshot:
            aid = snapshot.get("assembly_id")
            self._set_status(f"Assembly {aid} — click Preview assembly", ok=None)
        else:
            configure_preview_label(
                self._thumb_label,
                "empty",
                detail="No Assembly loaded — generate or load one first",
                width=PREVIEW_THUMB_MD,
                height=PREVIEW_THUMB_MD,
            )
            self._thumb_photo = None

    def on_preview(self) -> None:
        if not self._snapshot:
            self._set_status("Generate or load an assembly snapshot first.", ok=None)
            self._on_log("preview skipped — no snapshot loaded")
            return
        if self._preview_btn is not None:
            self._preview_btn.configure(text="⟳ Opening preview…")
        apply_status_atom(self._status_lbl, self._status_var, "working", detail="Opening interactive 3D…")
        if self._start_job:
            snapshot = self._snapshot

            def worker(_cancel) -> JobResult:
                try:
                    result = assembly_preview.preview_assembly_from_dict(snapshot, open_browser=True)
                except Exception as exc:  # noqa: BLE001
                    return JobResult(False, f"Preview failed: {exc}", detail=str(exc))
                return JobResult(True, "Preview OK", data={"result": result})

            def on_done(record: JobRecord) -> None:
                if self._preview_btn is not None:
                    self._preview_btn.configure(text="Preview assembly")
                if record.result and record.result.ok and record.result.data:
                    self._apply_preview_result(record.result.data["result"])
                else:
                    msg = record.result.message if record.result else "Preview failed"
                    self._set_status(msg, ok=False)

            self._start_job("Assembly preview", worker, on_done=on_done)
            return
        self._run_preview_sync()

    def _run_preview_sync(self) -> None:
        if self._preview_btn is not None:
            self._preview_btn.configure(text="Preview assembly")
        if not self._snapshot:
            return
        aid = self._snapshot.get("assembly_id", "?")
        placements = len(self._snapshot.get("module_placements") or [])
        self._on_log(f"preview start {aid} · {placements} placements")
        try:
            result = assembly_preview.preview_assembly_from_dict(self._snapshot, open_browser=True)
        except Exception as exc:  # noqa: BLE001
            self._on_log(f"preview error: {exc}")
            self._set_status(f"Preview failed: {exc}", ok=False)
            return
        self._apply_preview_result(result)

    def _apply_preview_result(self, result: dict) -> None:
        self._last_result = result
        url = str(result.get("preview_url") or "")
        self._url_var.set(url)
        assembly_preview.write_aps_preview_002_witness(result)
        mode = result.get("mode", "?")
        loaded = result.get("modules_loaded", 0)
        missing = result.get("missing_glb") or []
        profiles = ", ".join(result.get("material_profiles_sample") or []) or "—"
        if url:
            self._set_status(
                f"Interactive 3D opened in browser · {mode} · {loaded} modules · profiles: {profiles}",
                ok=True,
            )
        else:
            self._set_status(
                f"{mode} · {loaded} modules · profiles: {profiles} · no URL (use Open URL if needed)",
                ok=None if missing else True,
            )
        self._on_log(f"preview done {result.get('assembly_id')} · {mode} · url={url or '—'}")
        self._load_thumbnail(result.get("png") or "")
        if self._on_preview_thumb and result.get("png"):
            self._on_preview_thumb(result.get("png"), result)

    def on_open_url(self) -> None:
        url = self._url_var.get().strip()
        if not url.startswith("http"):
            self._set_status(
                "Run Preview assembly first — needs a loaded snapshot with resolved GLB paths.",
                ok=None,
            )
            self._on_log("open url skipped — no preview_url")
            return
        import webbrowser

        self._on_log(f"open url {url}")
        webbrowser.open(url)

    def on_copy_url(self) -> None:
        url = self._url_var.get().strip()
        if not url.startswith("http"):
            self._set_status("No URL yet — run Preview assembly first.", ok=None)
            return
        self.clipboard_clear()
        self.clipboard_append(url)
        self._on_log(f"copied url {url}")
        self._set_status(f"Copied: {url}", ok=True)

    def _load_thumbnail(self, png_rel: str) -> None:
        if not png_rel or Image is None or ImageTk is None:
            return
        path = repo_root() / str(png_rel).replace("\\", "/")
        if not path.is_file():
            self._on_log(f"preview thumb missing {path.name}")
            configure_preview_label(
                self._thumb_label,
                "error",
                detail="Thumbnail unavailable",
                hint="use Open in browser",
                width=PREVIEW_THUMB_MD,
                height=PREVIEW_THUMB_MD,
            )
            return
        try:
            img = Image.open(path).convert("RGB")
        except Exception as exc:  # noqa: BLE001
            self._on_log(f"preview thumb unreadable {path.name}: {exc}")
            configure_preview_label(
                self._thumb_label,
                "error",
                detail="Thumbnail unreadable",
                hint="use Open in browser",
                width=PREVIEW_THUMB_MD,
                height=PREVIEW_THUMB_MD,
            )
            return
        if image_is_near_black(img):
            self._thumb_photo = None
            configure_preview_label(
                self._thumb_label,
                "error",
                detail="Thumbnail unavailable",
                hint="use Open in browser",
                width=PREVIEW_THUMB_MD,
                height=PREVIEW_THUMB_MD,
            )
            self._on_log(f"preview thumb blank/black {path.name} — kept browser URL")
            return
        img.thumbnail((200, 200), Image.Resampling.LANCZOS)
        self._thumb_photo = ImageTk.PhotoImage(img)
        apply_preview_photo(self._thumb_label, self._thumb_photo)
