"""
PyQt5 ports of legacy `utils/asset_tools` helpers (`ClickableComboBox`, spin variants, labels).

Use these from any page; they do not depend on PySide6.
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any, Iterable, List, Optional, Union

from PyQt5.QtCore import QEvent, Qt
from PyQt5.QtGui import QFontMetrics, QStandardItem, QStandardItemModel


def create_widget_with_label(widget: QWidget, label_text: str) -> QWidget:
    """Stack a caption above ``widget`` (legacy ``subsystems.create_widget_with_label``)."""
    container = QWidget()
    layout = QVBoxLayout(container)
    layout.setContentsMargins(5, 0, 5, 0)
    layout.setSpacing(0)
    if label_text:
        label = QLabel(label_text)
        layout.addWidget(label, alignment=Qt.AlignTop)
    layout.addWidget(widget, alignment=Qt.AlignTop)
    return container


def populate_combobox_from_file_regex(
    file_path: Union[str, Path],
    pattern: str,
    combobox: QComboBox,
    *,
    clear: bool = True,
) -> int:
    """
    Read ``file_path`` as text, ``re.search`` the first group, split commas, strip ``;`` suffixes.
    Returns number of items added (legacy ``populate_from_config``).
    """
    path = Path(file_path)
    if not path.is_file():
        return 0
    text = path.read_text(encoding="utf-8", errors="replace")
    match = re.search(pattern, text)
    if not match:
        return 0
    raw = match.group(1).split(",")
    values = [v.split(";")[0].strip() for v in raw if v.strip()]
    if clear:
        combobox.clear()
    for value in values:
        combobox.addItem(value)
    return len(values)


def apply_readonly_spin(spin: QWidget, read_only: bool = True) -> None:
    """Hide spin arrows and set read-only (legacy ArrowHideableSpinBox behavior)."""
    from PyQt5.QtWidgets import QSpinBox

    if not isinstance(spin, QSpinBox):
        return
    spin.setReadOnly(read_only)
    spin.setButtonSymbols(
        QAbstractSpinBox.NoButtons if read_only else QAbstractSpinBox.UpDownArrows
    )


class ClickableComboBox(QComboBox):
    """
    Multi-select style combo: checkbox items, read-only line edit, click line edit toggles popup.
    Port of legacy PySide6 widget.
    """

    class _Delegate(QStyledItemDelegate):
        def sizeHint(self, option, index):
            size = super().sizeHint(option, index)
            size.setHeight(20)
            return size

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setModel(QStandardItemModel(self))
        self._close_on_line_edit_click = False
        self.setEditable(True)
        self.lineEdit().setReadOnly(True)
        self.setItemDelegate(ClickableComboBox._Delegate())
        self.model().dataChanged.connect(self._update_text)
        self.lineEdit().installEventFilter(self)
        self.view().viewport().installEventFilter(self)

    def resizeEvent(self, event):
        self._update_text()
        super().resizeEvent(event)

    def eventFilter(self, obj, event):
        if obj is self.lineEdit():
            if event.type() == QEvent.MouseButtonRelease:
                if self._close_on_line_edit_click:
                    self.hidePopup()
                else:
                    self.showPopup()
                return True
            return False
        if obj is self.view().viewport():
            if event.type() == QEvent.MouseButtonRelease:
                index = self.view().indexAt(event.pos())
                item = self.model().item(index.row())
                if item is None:
                    return False
                if item.checkState() == Qt.Checked:
                    item.setCheckState(Qt.Unchecked)
                else:
                    item.setCheckState(Qt.Checked)
                return True
            return False
        return super().eventFilter(obj, event)

    def showPopup(self):
        super().showPopup()

    def hidePopup(self):
        super().hidePopup()
        self.startTimer(100)
        self._update_text()

    def timerEvent(self, event):
        self.killTimer(event.timerId())
        self._close_on_line_edit_click = False

    def _update_text(self):
        texts: list[str] = []
        for i in range(self.model().rowCount()):
            it = self.model().item(i)
            if it and it.checkState() == Qt.Checked:
                texts.append(it.text())
        joined = ", ".join(texts)
        metrics = QFontMetrics(self.lineEdit().font())
        elided = metrics.elidedText(joined, Qt.ElideRight, self.lineEdit().width())
        self.lineEdit().setText(elided)

    def addItem(self, text, userData=None):  # noqa: A003 - Qt API
        item = QStandardItem()
        item.setText(text)
        if userData is not None:
            item.setData(userData, Qt.UserRole)
        else:
            item.setData(text, Qt.UserRole)
        item.setFlags(Qt.ItemIsEnabled | Qt.ItemIsUserCheckable)
        item.setData(Qt.Unchecked, Qt.CheckStateRole)
        self.model().appendRow(item)

    def addItems(self, texts: Iterable[str], datalist: Optional[List[Any]] = None) -> None:  # noqa: A003
        for i, text in enumerate(texts):
            data = None
            if datalist is not None and i < len(datalist):
                data = datalist[i]
            self.addItem(text, data)

    def currentData(self):  # noqa: A003 - Qt naming
        res = []
        for i in range(self.model().rowCount()):
            it = self.model().item(i)
            if it and it.checkState() == Qt.Checked:
                res.append(it.data(Qt.UserRole))
        return res
