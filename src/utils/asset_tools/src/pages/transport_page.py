"""Road / rail JSON workspace — lists `assets/configs/roads` and `assets/configs/rails`."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Optional

from PyQt5.QtCore import Qt, QUrl
from PyQt5.QtGui import QDesktopServices
from PyQt5.QtWidgets import (
    QFileDialog,
    QHBoxLayout,
    QLabel,
    QListWidget,
    QListWidgetItem,
    QMessageBox,
    QTabWidget,
    QTextEdit,
    QVBoxLayout,
    QWidget,
)

from qfluentwidgets import FluentIcon, PushButton

from .. import repo_paths


class TransportPage(QWidget):
    """Edit road & rail blueprint JSON next to engine structs (see `repo_paths`)."""

    assetConfigChanged = pyqtSignal(dict)

    def __init__(self, parent=None):
        super().__init__(parent)
        self._current_road: Optional[Path] = None
        self._current_rail: Optional[Path] = None
        self._build_ui()
        self._refresh_lists()

    def _build_ui(self) -> None:
        layout = QVBoxLayout(self)
        title = QLabel("Roads & rails — JSON examples")
        title.setStyleSheet("font-size: 18px; font-weight: bold; color: #5dca31;")
        layout.addWidget(title)

        hint = QLabel(
            "Files live under <b>assets/configs/roads</b> and <b>assets/configs/rails</b>. "
            "Building ↔ type index: <b>assets/configs/buildings/_building_types_index.json</b>."
        )
        hint.setTextFormat(Qt.RichText)
        hint.setWordWrap(True)
        layout.addWidget(hint)

        tabs = QTabWidget()
        tabs.addTab(self._make_split_panel("roads"), "Roads")
        tabs.addTab(self._make_split_panel("rails"), "Rails")
        layout.addWidget(tabs)

        idx_btn = PushButton("Open building types index")
        idx_btn.setIcon(FluentIcon.FOLDER)
        idx_btn.clicked.connect(self._open_building_index)
        layout.addWidget(idx_btn)

    def _make_split_panel(self, kind: str) -> QWidget:
        w = QWidget()
        h = QHBoxLayout(w)

        list_w = QListWidget()
        list_w.setMinimumWidth(260)
        text = QTextEdit()
        text.setPlaceholderText("{ … json … }")

        if kind == "roads":
            self.road_list = list_w
            self.road_editor = text
            list_w.currentItemChanged.connect(lambda _c, _p: self._load_editor("roads"))
            new_btn = PushButton("New from example")
            new_btn.clicked.connect(lambda: self._new_from_example("roads"))
            save_btn = PushButton("Save")
            save_btn.setIcon(FluentIcon.SAVE)
            save_btn.clicked.connect(lambda: self._save_current("roads"))
            ref_btn = PushButton("Refresh list")
            ref_btn.clicked.connect(self._refresh_lists)
        else:
            self.rail_list = list_w
            self.rail_editor = text
            list_w.currentItemChanged.connect(lambda _c, _p: self._load_editor("rails"))
            new_btn = PushButton("New from example")
            new_btn.clicked.connect(lambda: self._new_from_example("rails"))
            save_btn = PushButton("Save")
            save_btn.setIcon(FluentIcon.SAVE)
            save_btn.clicked.connect(lambda: self._save_current("rails"))
            ref_btn = PushButton("Refresh list")
            ref_btn.clicked.connect(self._refresh_lists)

        side = QWidget()
        sv = QVBoxLayout(side)
        sv.addWidget(list_w)
        row = QHBoxLayout()
        row.addWidget(new_btn)
        row.addWidget(save_btn)
        row.addWidget(ref_btn)
        sv.addLayout(row)

        h.addWidget(side)
        h.addWidget(text, 1)
        return w

    def _refresh_lists(self) -> None:
        self._fill_list(self.road_list, repo_paths.roads_configs_dir)
        self._fill_list(self.rail_list, repo_paths.rails_configs_dir)

    @staticmethod
    def _fill_list(list_w: QListWidget, directory: Path) -> None:
        list_w.clear()
        directory.mkdir(parents=True, exist_ok=True)
        for p in sorted(directory.glob("*.json")):
            list_w.addItem(QListWidgetItem(str(p)))

    def _load_editor(self, kind: str) -> None:
        if kind == "roads":
            lw, editor = self.road_list, self.road_editor
            self._current_road = None
        else:
            lw, editor = self.rail_list, self.rail_editor
            self._current_rail = None
        item = lw.currentItem()
        if not item:
            editor.clear()
            return
        path = Path(item.text())
        try:
            editor.setPlainText(path.read_text(encoding="utf-8"))
            if kind == "roads":
                self._current_road = path
            else:
                self._current_rail = path
        except OSError as e:
            editor.setPlainText(f"/* read error: {e} */")

    def _new_from_example(self, kind: str) -> None:
        if kind == "roads":
            base = repo_paths.roads_configs_dir / "example_road_segment_v1.json"
            editor = self.road_editor
            listings = self.road_list
            ext_current = "_current_road"
        else:
            base = repo_paths.rails_configs_dir / "example_rail_track_v1.json"
            editor = self.rail_editor
            listings = self.rail_list
            ext_current = "_current_rail"
        if not base.is_file():
            QMessageBox.warning(self, "Missing example", f"Example not found:\n{base}")
            return
        text = base.read_text(encoding="utf-8")
        try:
            obj = json.loads(text)
            if "id" in obj:
                obj["id"] = obj["id"] + "_copy"
            text = json.dumps(obj, indent=2)
        except json.JSONDecodeError:
            pass
        editor.setPlainText(text)
        setattr(self, ext_current, None)

    def _save_current(self, kind: str) -> None:
        if kind == "roads":
            directory = repo_paths.roads_configs_dir
            editor = self.road_editor
            current_attr = "_current_road"
        else:
            directory = repo_paths.rails_configs_dir
            editor = self.rail_editor
            current_attr = "_current_rail"
        directory.mkdir(parents=True, exist_ok=True)
        path = getattr(self, current_attr)
        body = editor.toPlainText()
        try:
            json.loads(body)
        except json.JSONDecodeError as e:
            QMessageBox.critical(self, "Invalid JSON", str(e))
            return
        if path is None:
            name, _ = QFileDialog.getSaveFileName(
                self, "Save JSON", str(directory), "JSON (*.json)"
            )
            if not name:
                return
            path = Path(name)
        try:
            path.write_text(body, encoding="utf-8")
            setattr(self, current_attr, path)
            self._refresh_lists()
            QMessageBox.information(self, "Saved", str(path))
        except OSError as e:
            QMessageBox.critical(self, "Save failed", str(e))

    def _open_building_index(self) -> None:
        p = repo_paths.building_types_index_json
        if not p.is_file():
            QMessageBox.warning(self, "Missing index", str(p))
            return
        QDesktopServices.openUrl(QUrl.fromLocalFile(str(p.resolve())))

