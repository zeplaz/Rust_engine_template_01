"""World generation tuning — edit `assets/config/world_gen_tuning.json` (same file as Bevy / F8 UI)."""

from __future__ import annotations

import copy
import json
import shutil
from pathlib import Path

from PyQt5.QtCore import Qt, QUrl
from PyQt5.QtGui import QDesktopServices
from PyQt5.QtWidgets import (
    QHBoxLayout,
    QLabel,
    QMessageBox,
    QTabWidget,
    QTextEdit,
    QVBoxLayout,
    QWidget,
)

from qfluentwidgets import FluentIcon, PushButton

from .. import repo_paths

TAB_OVERVIEW = 0
TAB_NOISE = 1
TAB_BIOME = 2
TAB_FULL = 3


class WorldGenPage(QWidget):
    """Desktop editor for noise + biome tuning overlay consumed by the engine."""

    def __init__(self, parent=None) -> None:
        super().__init__(parent)
        self._working: dict = {}
        self._last_tab: int | None = None
        self._build()

    def _paths(self) -> tuple[Path, Path]:
        return repo_paths.world_gen_tuning_json, repo_paths.world_gen_tuning_example_json

    def _build(self) -> None:
        layout = QVBoxLayout(self)
        title = QLabel("World generation — tuning JSON")
        title.setStyleSheet("font-size: 18px; font-weight: bold; color: #5dca31;")
        layout.addWidget(title)

        self.tabs = QTabWidget()
        self._overview = QWidget()
        self._setup_overview_tab()
        self.tabs.addTab(self._overview, "Overview")

        self.noise_editor = QTextEdit()
        self.noise_editor.setPlaceholderText(
            '"noise_sampling": { "warp_noise_scale_mul": 0.25, ... }\n(edit the object only)'
        )
        self.noise_editor.setMinimumHeight(320)
        noise_wrap = QWidget()
        nlay = QVBoxLayout(noise_wrap)
        nhint = QLabel(
            "Edit the <b>noise_sampling</b> object only. "
            "Switching tabs validates and merges into the working copy."
        )
        nhint.setWordWrap(True)
        nlay.addWidget(nhint)
        nlay.addWidget(self.noise_editor, 1)
        self.tabs.addTab(noise_wrap, "Noise sampling")

        self.biome_editor = QTextEdit()
        self.biome_editor.setPlaceholderText(
            '"biome_tuning": { "sea_level": 0.4, ... }\n(edit the object only)'
        )
        self.biome_editor.setMinimumHeight(320)
        biome_wrap = QWidget()
        blay = QVBoxLayout(biome_wrap)
        bhint = QLabel(
            "Edit the <b>biome_tuning</b> object (all Whittaker weights + thresholds). "
            "Matches <code>BiomeTuning</code> in Rust — see matrix §8 vs egui subset."
        )
        bhint.setWordWrap(True)
        blay.addWidget(bhint)
        blay.addWidget(self.biome_editor, 1)
        self.tabs.addTab(biome_wrap, "Biome tuning")

        self.full_editor = QTextEdit()
        self.full_editor.setPlaceholderText(
            '{\n  "noise_sampling": { ... },\n  "biome_tuning": { ... }\n}'
        )
        self.full_editor.setMinimumHeight(320)
        full_wrap = QWidget()
        flay = QVBoxLayout(full_wrap)
        fhint = QLabel("Full <code>WorldGenTuningOverlay</code> JSON root (both keys).")
        fhint.setWordWrap(True)
        flay.addWidget(fhint)
        flay.addWidget(self.full_editor, 1)
        self.tabs.addTab(full_wrap, "Full JSON")

        self.tabs.currentChanged.connect(self._on_tab_changed)
        layout.addWidget(self.tabs, 1)

        row1 = QHBoxLayout()
        b_load_active = PushButton("Load active")
        b_load_active.setIcon(FluentIcon.FOLDER)
        b_load_active.clicked.connect(self._load_active)
        b_load_example = PushButton("Load example")
        b_load_example.clicked.connect(self._load_example)
        b_save = PushButton("Save active")
        b_save.setIcon(FluentIcon.SAVE)
        b_save.clicked.connect(self._save_active)
        b_copy = PushButton("Copy example → active (overwrite)")
        b_copy.clicked.connect(self._copy_example_to_active)
        row1.addWidget(b_load_active)
        row1.addWidget(b_load_example)
        row1.addWidget(b_save)
        row1.addWidget(b_copy)
        layout.addLayout(row1)

        self.status = QLabel("")
        self.status.setWordWrap(True)
        self.status.setStyleSheet("color: #88aa88; font-size: 12px;")
        layout.addWidget(self.status)

        self._last_tab = self.tabs.currentIndex()
        self._load_active_or_hint()

    def _setup_overview_tab(self) -> None:
        v = QVBoxLayout(self._overview)
        active, example = self._paths()
        hint = QLabel(
            f"The game reads <b>{active.relative_to(repo_paths.REPO_ROOT)}</b> when present "
            f"(see <code>WORLD_GEN_TUNING_JSON_PATH</code> in Rust). "
            f"Start from the committed example <b>{example.name}</b>. "
            "<br><br>Use sub-tabs <b>Noise sampling</b> and <b>Biome tuning</b> for focused edits; "
            "<b>Full JSON</b> replaces the whole overlay. "
            "Switching tabs validates JSON for the leaving tab."
            "<br>In-game: <b>F8</b> egui world generator. "
            "Headless: <code>cargo run --bin world_generator</code>."
        )
        hint.setTextFormat(Qt.RichText)
        hint.setWordWrap(True)
        v.addWidget(hint)

        row_docs = QHBoxLayout()
        b_pipe = PushButton("Pipeline spec (01)")
        b_pipe.clicked.connect(self._open_pipeline_spec)
        b_topic = PushButton("Layered gen + preview (designer)")
        b_topic.clicked.connect(self._open_composite_topic)
        b_matrix = PushButton("Preview integration matrix")
        b_matrix.clicked.connect(self._open_preview_matrix)
        row_docs.addWidget(b_pipe)
        row_docs.addWidget(b_topic)
        row_docs.addWidget(b_matrix)
        row_docs.addStretch()
        v.addLayout(row_docs)
        v.addStretch()

    def _sync_editors_from_working(self) -> None:
        ns = self._working.get("noise_sampling", {})
        bt = self._working.get("biome_tuning", {})
        self.noise_editor.setPlainText(json.dumps(ns, indent=2) + "\n")
        self.biome_editor.setPlainText(json.dumps(bt, indent=2) + "\n")
        self.full_editor.setPlainText(json.dumps(self._working, indent=2) + "\n")

    def _on_tab_changed(self, idx: int) -> None:
        prev = self._last_tab
        if prev is not None and prev != idx:
            if not self._flush_tab_to_working(prev):
                self.tabs.blockSignals(True)
                self.tabs.setCurrentIndex(prev)
                self.tabs.blockSignals(False)
                return
        self._refresh_tab_from_working(idx)
        self._last_tab = idx

    def _flush_tab_to_working(self, tab: int) -> bool:
        if tab == TAB_OVERVIEW:
            return True
        if tab == TAB_NOISE:
            return self._merge_noise_from_editor()
        if tab == TAB_BIOME:
            return self._merge_biome_from_editor()
        if tab == TAB_FULL:
            return self._replace_working_from_full_editor()
        return True

    def _merge_noise_from_editor(self) -> bool:
        raw = self.noise_editor.toPlainText().strip()
        if not raw:
            self._working["noise_sampling"] = {}
            return True
        try:
            data = json.loads(raw)
        except json.JSONDecodeError as e:
            QMessageBox.critical(self, "Invalid JSON (noise)", str(e))
            return False
        if not isinstance(data, dict):
            QMessageBox.critical(self, "Invalid noise", "Root must be a JSON object.")
            return False
        self._working["noise_sampling"] = data
        self.full_editor.setPlainText(json.dumps(self._working, indent=2) + "\n")
        return True

    def _merge_biome_from_editor(self) -> bool:
        raw = self.biome_editor.toPlainText().strip()
        if not raw:
            self._working["biome_tuning"] = {}
            return True
        try:
            data = json.loads(raw)
        except json.JSONDecodeError as e:
            QMessageBox.critical(self, "Invalid JSON (biome)", str(e))
            return False
        if not isinstance(data, dict):
            QMessageBox.critical(self, "Invalid biome", "Root must be a JSON object.")
            return False
        self._working["biome_tuning"] = data
        self.full_editor.setPlainText(json.dumps(self._working, indent=2) + "\n")
        return True

    def _replace_working_from_full_editor(self) -> bool:
        raw = self.full_editor.toPlainText().strip()
        if not raw:
            QMessageBox.warning(self, "Empty", "Full JSON is empty.")
            return False
        try:
            data = json.loads(raw)
        except json.JSONDecodeError as e:
            QMessageBox.critical(self, "Invalid JSON (full)", str(e))
            return False
        if not isinstance(data, dict):
            QMessageBox.critical(self, "Invalid full", "Root must be a JSON object.")
            return False
        self._working = copy.deepcopy(data)
        return True

    def _refresh_tab_from_working(self, tab: int) -> None:
        if tab == TAB_NOISE:
            self.noise_editor.setPlainText(
                json.dumps(self._working.get("noise_sampling", {}), indent=2) + "\n"
            )
        elif tab == TAB_BIOME:
            self.biome_editor.setPlainText(
                json.dumps(self._working.get("biome_tuning", {}), indent=2) + "\n"
            )
        elif tab == TAB_FULL:
            self.full_editor.setPlainText(json.dumps(self._working, indent=2) + "\n")

    def _ensure_flushed(self) -> bool:
        cur = self.tabs.currentIndex()
        return self._flush_tab_to_working(cur)

    def _load_active_or_hint(self) -> None:
        active, example = self._paths()
        if active.is_file():
            self._load_active()
        elif example.is_file():
            self.status.setText(
                f"No active file yet — loaded example for editing. "
                f"Save as {repo_paths.world_gen_tuning_json.name} when ready."
            )
            self._load_example()
        else:
            self.status.setText("Neither active nor example JSON found.")

    def _load_active(self) -> None:
        active, _ = self._paths()
        if not active.is_file():
            self.status.setText(f"Active file missing: {active}")
            self._working = {}
            self._sync_editors_from_working()
            return
        try:
            text = active.read_text(encoding="utf-8")
            self._working = json.loads(text) if text.strip() else {}
            if not isinstance(self._working, dict):
                raise ValueError("Root must be an object")
            self._sync_editors_from_working()
            self.status.setText(f"Loaded {active.name}")
        except (OSError, json.JSONDecodeError, ValueError) as e:
            self.status.setText(str(e))
            QMessageBox.critical(self, "Load failed", str(e))

    def _load_example(self) -> None:
        _, example = self._paths()
        if not example.is_file():
            self.status.setText(f"Example missing: {example}")
            return
        try:
            text = example.read_text(encoding="utf-8")
            self._working = json.loads(text)
            if not isinstance(self._working, dict):
                raise ValueError("Root must be an object")
            self._sync_editors_from_working()
            self.status.setText(f"Loaded {example.name} (example only)")
        except (OSError, json.JSONDecodeError, ValueError) as e:
            self.status.setText(str(e))
            QMessageBox.critical(self, "Load failed", str(e))

    def _save_active(self) -> None:
        if not self._ensure_flushed():
            return
        active, _ = self._paths()
        try:
            pretty = json.dumps(self._working, indent=2)
            json.loads(pretty)  # sanity
        except (TypeError, json.JSONDecodeError) as e:
            QMessageBox.critical(self, "Invalid state", str(e))
            return
        active.parent.mkdir(parents=True, exist_ok=True)
        try:
            active.write_text(pretty + "\n", encoding="utf-8")
            self.status.setText(f"Saved {active.name}")
        except OSError as e:
            QMessageBox.critical(self, "Save failed", str(e))

    def _copy_example_to_active(self) -> None:
        active, example = self._paths()
        if not example.is_file():
            QMessageBox.warning(self, "Missing example", str(example))
            return
        reply = QMessageBox.question(
            self,
            "Overwrite active",
            f"This will overwrite:\n{active}\nwith the example file. Continue?",
        )
        if reply != QMessageBox.Yes:
            return
        try:
            shutil.copyfile(example, active)
            self._load_active()
            self.status.setText(f"Copied example → {active.name}")
        except OSError as e:
            QMessageBox.critical(self, "Copy failed", str(e))

    def _open_pipeline_spec(self) -> None:
        spec = (
            repo_paths.REPO_ROOT
            / "prompts"
            / "designer_questions"
            / "terrain_world"
            / "spec"
            / "01_world_generation_pipeline.md"
        )
        self._open_local_file(spec)

    def _open_composite_topic(self) -> None:
        spec = (
            repo_paths.REPO_ROOT
            / "prompts"
            / "designer_questions"
            / "terrain_world"
            / "composite_style_worldgen_v1.md"
        )
        self._open_local_file(spec)

    def _open_preview_matrix(self) -> None:
        spec = (
            repo_paths.REPO_ROOT
            / "prompts"
            / "matrix"
            / "terrain_biome"
            / "composite_style_preview_integration_matrix_v1.md"
        )
        self._open_local_file(spec)

    def _open_local_file(self, path: Path) -> None:
        if path.is_file():
            QDesktopServices.openUrl(QUrl.fromLocalFile(str(path.resolve())))
        else:
            QMessageBox.information(self, "Not found", str(path))
