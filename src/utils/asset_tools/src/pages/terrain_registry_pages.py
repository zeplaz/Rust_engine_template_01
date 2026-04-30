"""Terrain material registry, tag registry, and material rules (JSON + RON) — see prompts designer docs."""

from __future__ import annotations

import json
import shutil
from pathlib import Path
from typing import Callable

from PyQt5.QtCore import Qt, QUrl
from PyQt5.QtGui import QDesktopServices
from PyQt5.QtWidgets import (
    QHBoxLayout,
    QLabel,
    QMessageBox,
    QTextEdit,
    QVBoxLayout,
    QWidget,
)

from qfluentwidgets import FluentIcon, PushButton

from .. import repo_paths


def _open_markdown_under_prompts(relative_parts: tuple[str, ...]) -> None:
    spec = repo_paths.REPO_ROOT.joinpath("prompts", *relative_parts)
    if spec.is_file():
        QDesktopServices.openUrl(QUrl.fromLocalFile(str(spec.resolve())))
    else:
        QMessageBox.information(None, "Not found", str(spec))


class JsonTerrainConfigPage(QWidget):
    """Editor for a JSON root object with a required list key (e.g. materials, tags)."""

    def __init__(
        self,
        parent,
        title: str,
        active_path: Path,
        example_path: Path,
        list_key: str,
        placeholder: str,
        doc_hint: str,
        doc_path: tuple[str, ...],
        validate_row: Callable[[dict], str | None] | None = None,
    ) -> None:
        super().__init__(parent)
        self._active = active_path
        self._example = example_path
        self._list_key = list_key
        self._doc_path = doc_path
        self._validate_row = validate_row
        self._build(title, placeholder, doc_hint)

    def _build(self, title: str, placeholder: str, doc_hint: str) -> None:
        layout = QVBoxLayout(self)
        tl = QLabel(title)
        tl.setStyleSheet("font-size: 18px; font-weight: bold; color: #5dca31;")
        layout.addWidget(tl)
        hint = QLabel(doc_hint)
        hint.setTextFormat(Qt.RichText)
        hint.setWordWrap(True)
        layout.addWidget(hint)
        self.editor = QTextEdit()
        self.editor.setPlaceholderText(placeholder)
        self.editor.setMinimumHeight(360)
        layout.addWidget(self.editor, 1)
        row1 = QHBoxLayout()
        for label, fn in (
            ("Load active", self._load_active),
            ("Load example", self._load_example),
            ("Save active", self._save_active),
            ("Copy example → active", self._copy_example),
        ):
            b = PushButton(label)
            if "Save" in label:
                b.setIcon(FluentIcon.SAVE)
            elif "Load active" in label:
                b.setIcon(FluentIcon.FOLDER)
            b.clicked.connect(fn)
            row1.addWidget(b)
        layout.addLayout(row1)
        row2 = QHBoxLayout()
        bd = PushButton("Open designer doc")
        bd.clicked.connect(lambda: _open_markdown_under_prompts(self._doc_path))
        row2.addWidget(bd)
        b_matrix = PushButton("Open unification matrix")

        def _matrix() -> None:
            p = (
                repo_paths.REPO_ROOT
                / "prompts"
                / "matrix"
                / "terrain_biome"
                / "material_unification_matrix_v1.md"
            )
            if p.is_file():
                QDesktopServices.openUrl(QUrl.fromLocalFile(str(p.resolve())))
            else:
                QMessageBox.information(self, "Not found", str(p))

        b_matrix.clicked.connect(_matrix)
        row2.addWidget(b_matrix)
        row2.addStretch()
        layout.addLayout(row2)
        self.status = QLabel("")
        self.status.setWordWrap(True)
        self.status.setStyleSheet("color: #88aa88; font-size: 12px;")
        layout.addWidget(self.status)
        self._load_active_or_example()

    def _load_active_or_example(self) -> None:
        if self._active.is_file():
            self._load_active()
        elif self._example.is_file():
            self.status.setText(
                f"No active file — loaded example. Save as {self._active.name} when ready."
            )
            self._load_example()
        else:
            self.status.setText("Neither active nor example found.")

    def _load_active(self) -> None:
        if not self._active.is_file():
            self.status.setText(f"Active missing: {self._active}")
            self.editor.clear()
            return
        try:
            self.editor.setPlainText(self._active.read_text(encoding="utf-8"))
            self.status.setText(f"Loaded {self._active.name}")
        except OSError as e:
            self.status.setText(str(e))

    def _load_example(self) -> None:
        if not self._example.is_file():
            self.status.setText(f"Example missing: {self._example}")
            return
        try:
            self.editor.setPlainText(self._example.read_text(encoding="utf-8"))
            self.status.setText(f"Loaded {self._example.name} (example)")
        except OSError as e:
            self.status.setText(str(e))

    def _validate_root(self, data: dict) -> str | None:
        if self._list_key not in data:
            return f"Root must include '{self._list_key}' array"
        if not isinstance(data[self._list_key], list):
            return f"'{self._list_key}' must be a JSON array"
        if self._validate_row:
            for i, row in enumerate(data[self._list_key]):
                if not isinstance(row, dict):
                    return f"Row {i} must be an object"
                err = self._validate_row(row)
                if err:
                    return f"Row {i}: {err}"
        return None

    def _save_active(self) -> None:
        body = self.editor.toPlainText()
        try:
            data = json.loads(body)
            if not isinstance(data, dict):
                raise ValueError("Root must be a JSON object")
            err = self._validate_root(data)
            if err:
                raise ValueError(err)
        except (json.JSONDecodeError, ValueError) as e:
            QMessageBox.critical(self, "Invalid JSON", str(e))
            return
        self._active.parent.mkdir(parents=True, exist_ok=True)
        try:
            pretty = json.dumps(data, indent=2)
            self._active.write_text(pretty + "\n", encoding="utf-8")
            self.status.setText(f"Saved {self._active.name}")
        except OSError as e:
            QMessageBox.critical(self, "Save failed", str(e))

    def _copy_example(self) -> None:
        if not self._example.is_file():
            QMessageBox.warning(self, "Missing example", str(self._example))
            return
        reply = QMessageBox.question(
            self,
            "Overwrite active",
            f"Overwrite:\n{self._active}\nwith example?",
        )
        if reply != QMessageBox.Yes:
            return
        try:
            shutil.copyfile(self._example, self._active)
            self._load_active()
            self.status.setText(f"Copied example → {self._active.name}")
        except OSError as e:
            QMessageBox.critical(self, "Copy failed", str(e))


def _validate_material_row(row: dict) -> str | None:
    if "name" not in row or not isinstance(row["name"], str):
        return "missing string 'name'"
    if "family" not in row or not isinstance(row["family"], str):
        return "missing string 'family' (TerrainClass name)"
    return None


def _validate_tag_row(row: dict) -> str | None:
    if "name" not in row or not isinstance(row["name"], str):
        return "missing string 'name'"
    return None


class MaterialRegistryPage(JsonTerrainConfigPage):
    def __init__(self, parent=None) -> None:
        super().__init__(
            parent,
            title="Terrain — material registry (JSON)",
            active_path=repo_paths.material_registry_json,
            example_path=repo_paths.material_registry_example_json,
            list_key="materials",
            placeholder='{\n  "schema_version": 1,\n  "materials": [ { "name": "...", "family": "Grassland", ... } ]\n}',
            doc_hint=(
                f"Active: <b>{repo_paths.material_registry_json.relative_to(repo_paths.REPO_ROOT)}</b>. "
                "<code>family</code> must match a <code>TerrainClass</code> variant "
                "(see Rust <code>src/terrain/biome.rs</code>). "
                "Paired docs: material tag rule system + unification matrix."
            ),
            doc_path=("designer_questions", "terrain_world", "material_tag_rule_system_v1.md"),
            validate_row=_validate_material_row,
        )


class TagRegistryPage(JsonTerrainConfigPage):
    def __init__(self, parent=None) -> None:
        super().__init__(
            parent,
            title="Terrain — tag registry (JSON)",
            active_path=repo_paths.tag_registry_json,
            example_path=repo_paths.tag_registry_example_json,
            list_key="tags",
            placeholder='{\n  "schema_version": 1,\n  "tags": [ { "name": "lowland", "category": "terrain" } ]\n}',
            doc_hint=(
                f"Active: <b>{repo_paths.tag_registry_json.relative_to(repo_paths.REPO_ROOT)}</b>. "
                "Tag names are interned to <code>TagId</code> at engine load (see matrix)."
            ),
            doc_path=("designer_questions", "terrain_world", "material_tag_rule_system_v1.md"),
            validate_row=_validate_tag_row,
        )


class MaterialRulesPage(QWidget):
    """RON text editor for material_rules.ron (engine parses with serde + RON)."""

    def __init__(self, parent=None) -> None:
        super().__init__(parent)
        self._active = repo_paths.material_rules_ron
        self._example = repo_paths.material_rules_example_ron
        layout = QVBoxLayout(self)
        tl = QLabel("Terrain — material rules (RON)")
        tl.setStyleSheet("font-size: 18px; font-weight: bold; color: #5dca31;")
        layout.addWidget(tl)
        hint = QLabel(
            f"Active: <b>{self._active.relative_to(repo_paths.REPO_ROOT)}</b>. "
            "Hand-edited rule list: <code>required</code>, <code>forbidden</code>, "
            "<code>result</code> (must match a <code>MaterialDef.name</code>), "
            "<code>priority</code>. Full parser lives in Rust."
        )
        hint.setWordWrap(True)
        layout.addWidget(hint)
        self.editor = QTextEdit()
        self.editor.setPlaceholderText("( schema_version: 1, rules: [ ... ], )")
        self.editor.setMinimumHeight(360)
        layout.addWidget(self.editor, 1)
        row1 = QHBoxLayout()
        for label, fn in (
            ("Load active", self._load_active),
            ("Load example", self._load_example),
            ("Save active", self._save_active),
            ("Copy example → active", self._copy_example),
        ):
            b = PushButton(label)
            if "Save" in label:
                b.setIcon(FluentIcon.SAVE)
            elif "Load active" in label:
                b.setIcon(FluentIcon.FOLDER)
            b.clicked.connect(fn)
            row1.addWidget(b)
        layout.addLayout(row1)
        bdoc = PushButton("Open designer doc")
        bdoc.clicked.connect(
            lambda: _open_markdown_under_prompts(
                ("designer_questions", "terrain_world", "material_tag_rule_system_v1.md")
            )
        )
        layout.addWidget(bdoc)
        self.status = QLabel("")
        self.status.setWordWrap(True)
        self.status.setStyleSheet("color: #88aa88; font-size: 12px;")
        layout.addWidget(self.status)
        self._load_active_or_example()

    def _load_active_or_example(self) -> None:
        if self._active.is_file():
            self._load_active()
        elif self._example.is_file():
            self.status.setText(
                f"No active file — loaded example. Save as {self._active.name} when ready."
            )
            self._load_example()
        else:
            self.status.setText("Neither active nor example found.")

    def _load_active(self) -> None:
        if not self._active.is_file():
            self.status.setText(f"Active missing: {self._active}")
            self.editor.clear()
            return
        try:
            self.editor.setPlainText(self._active.read_text(encoding="utf-8"))
            self.status.setText(f"Loaded {self._active.name}")
        except OSError as e:
            self.status.setText(str(e))

    def _load_example(self) -> None:
        if not self._example.is_file():
            self.status.setText(f"Example missing: {self._example}")
            return
        try:
            self.editor.setPlainText(self._example.read_text(encoding="utf-8"))
            self.status.setText(f"Loaded {self._example.name} (example)")
        except OSError as e:
            self.status.setText(str(e))

    def _save_active(self) -> None:
        body = self.editor.toPlainText().strip()
        if not body:
            QMessageBox.warning(self, "Empty", "Nothing to save.")
            return
        if "rules" not in body:
            reply = QMessageBox.question(
                self,
                "No 'rules' substring",
                "File does not contain the word 'rules'. Save anyway?",
            )
            if reply != QMessageBox.Yes:
                return
        self._active.parent.mkdir(parents=True, exist_ok=True)
        try:
            self._active.write_text(body + "\n", encoding="utf-8")
            self.status.setText(f"Saved {self._active.name}")
        except OSError as e:
            QMessageBox.critical(self, "Save failed", str(e))

    def _copy_example(self) -> None:
        if not self._example.is_file():
            QMessageBox.warning(self, "Missing example", str(self._example))
            return
        reply = QMessageBox.question(
            self,
            "Overwrite active",
            f"Overwrite:\n{self._active}\nwith example?",
        )
        if reply != QMessageBox.Yes:
            return
        try:
            shutil.copyfile(self._example, self._active)
            self._load_active()
            self.status.setText(f"Copied example → {self._active.name}")
        except OSError as e:
            QMessageBox.critical(self, "Copy failed", str(e))
