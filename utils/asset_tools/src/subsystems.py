import re
import PySide6 as py6      
from PySide6 import QtCore, QtGui, QtWidgets

def append_widget(parent, widget, title = None, layout = None):
    if isinstance(parent, py6.QtWidgets.QToolBox):
        return parent.addItem(widget, title)
    elif isinstance(parent, py6.QtWidgets.QLayout):
        parent.addWidget(widget)   
    elif isinstance(parent, py6.QtWidgets.QTabWidget):
        parent.addTab(widget, title)
    elif isinstance(parent, py6.QtWidgets.QWidget):
        if parent.layout() is not None:
            parent.layout().addWidget(widget)
        else:
            layout.addWidget(widget)
            parent.setLayout(parent.layout)
    else:
        raise ValueError("Parent must be a QToolBox or a QLayout or request adding type:{}".format(type(parent)))
    return None 

def populate_from_config(file_path, pattern, combobox):
    # 2. Read the File
    with open(file_path, "r") as file:
        content = file.read()

    # 3. Parse the Content
    # Using a regex pattern to capture values inside parentheses after ":Power:voltages:"
    match = re.search(pattern, content)
    if match:
        values = match.group(1).split(',')
        # Extracting values before ";unit" for clarity
        values = [v.split(';')[0] for v in values]

        # 4. Populate the QComboBox
        for value in values:
            combobox.addItem(value)


def create_widget_with_label(widget, label_text):
    container_widget = py6.QtWidgets.QWidget()
    layout = py6.QtWidgets.QVBoxLayout()
    layout.setContentsMargins(5, 0, 5, 0)  # Set margins to zero
    layout.setSpacing(0)  # Set spacing to zero
    if label_text:
        label = py6.QtWidgets.QLabel(label_text)
        layout.addWidget(label, alignment=py6.QtCore.Qt.AlignmentFlag.AlignTop)
    
    layout.addWidget(widget, alignment=py6.QtCore.Qt.AlignmentFlag.AlignTop)
    container_widget.setLayout(layout)
    return container_widget


class ArrowHideableSpinBox(QtWidgets.QSpinBox):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

    
    def toggleArrows(self, visible=True):
        """ Toggles the visibility of arrows. """
        if visible:
            self.setButtonSymbols(QtWidgets.QAbstractSpinBox.UpDownArrows)
        else:
            self.setButtonSymbols(QtWidgets.QAbstractSpinBox.NoButtons)

    def makeReadOnly(self, is_read_only=True):
        """ If is_read_only is True, make the spinbox read-only and hide arrows. """
        if is_read_only:
            self.setReadOnly(True)
            self.toggleArrows(False)
        else:
            self.setReadOnly(False)
            self.toggleArrows(True)

class ClickableComboBox(py6.QtWidgets.QComboBox):

    class Delegate(py6.QtWidgets.QStyledItemDelegate):

        def sizeHint(self, option, index):

            size = super().sizeHint(option, index)
            size.setHeight(20)
            return size

    def __init__(self, parent=None):
        super(ClickableComboBox, self).__init__(parent)
        self.clicked = False
        self.setEditable(True)
        self.lineEdit().setReadOnly(True)
        
        self.setItemDelegate(ClickableComboBox.Delegate())

        self.model().dataChanged.connect(self.update_text)

        self.lineEdit().installEventFilter(self)
        self.closeOnLineEditClick = False

        self.view().viewport().installEventFilter(self)

    def resizeEvent(self, event):
        self.update_text()
        super().resizeEvent(event)
    
    def eventFilter(self, object, event):
        if object == self.lineEdit():
            if event.type() == py6.QtCore.QEvent.MouseButtonRelease:
                if self.closeOnLineEditClick:

                    self.hidePopup()
                else:
                    self.showPopup()
                return True
            return False 
        if object == self.view().viewport():
            if event.type() == py6.QtCore.QEvent.MouseButtonRelease:
                index = self.view().indexAt(event.pos())
                item = self.model().item(index.row())
                if item.checkState() == py6.QtCore.Qt.Checked:
                    item.setCheckState(py6.QtCore.Qt.Unchecked)
                else:
                    item.setCheckState(py6.QtCore.Qt.Checked)
                return True
            return False
    def showPopup(self):
        super().showPopup()
        self.clickOnLineClick = True

    def hidePopup(self):
        super().hidePopup()
        self.startTimer(100)
        self.update_text()
    
    def timerEvent(self, event):
        self.killTimer(event.timerId())
        self.closeOnLineEditClick = False
        

    def update_text(self):
        text = []
        for i in range(self.model().rowCount()):
            if self.model().item(i).checkState() == py6.QtCore.Qt.Checked:
                text.append(self.model().item(i).text())
        text = ", ".join(text)

        metrics = py6.QtGui.QFontMetrics(self.lineEdit().font())
        elident_text = metrics.elidedText(text, py6.QtCore.Qt.ElideRight, self.lineEdit().width())
        self.lineEdit().setText(elident_text)
    
    def addItem(self, text, data=None):
        item = py6.QtGui.QStandardItem()
        item.setText(text)
        if data is not None:
            item.setData(data, py6.QtCore.Qt.UserRole)
        else:
            item.setData(text, py6.QtCore.Qt.UserRole)

        item.setFlags(py6.QtCore.Qt.ItemIsEnabled | py6.QtCore.Qt.ItemIsUserCheckable)
        item.setData(py6.QtCore.Qt.Unchecked, py6.QtCore.Qt.CheckStateRole)
        self.model().appendRow(item)
    
    def addItems(self, texts, datalist=None):
        
        for i, text in enumerate(texts):
            try:
                data = datalist[i]
            except (TypeError,IndexError):
                data = None
            self.addItem(text, data=data)
    
    def currentData(self):

        res = []    
        for i in range(self.model().rowCount()):
            if self.model().item(i).checkState() == py6.QtCore.Qt.Checked:
                            res.append(self.model().item(i).data(py6.QtCore.Qt.UserRole))
        return res




    
class TreeComboBox(py6.QtWidgets.QComboBox):
    def __init__(self, *args):
        super().__init__(*args)

        self.__skip_next_hide = False

        tree_view = QTreeView(self)
        tree_view.setFrameShape(QFrame.NoFrame)
        tree_view.setEditTriggers(tree_view.NoEditTriggers)
        tree_view.setAlternatingRowColors(True)
        tree_view.setSelectionBehavior(tree_view.SelectRows)
        tree_view.setWordWrap(True)
        tree_view.setAllColumnsShowFocus(True)
        self.setView(tree_view)

        self.view().viewport().installEventFilter(self)

    def showPopup(self):
        self.setRootModelIndex(QModelIndex())
        super().showPopup()

    def hidePopup(self):
        self.setRootModelIndex(self.view().currentIndex().parent())
        self.setCurrentIndex(self.view().currentIndex().row())
        if self.__skip_next_hide:
            self.__skip_next_hide = False
        else:
            super().hidePopup()

    def selectIndex(self, index):
        self.setRootModelIndex(index.parent())
        self.setCurrentIndex(index.row())

    def eventFilter(self, object, event):
        if event.type() == QEvent.MouseButtonPress and object is self.view().viewport():
            index = self.view().indexAt(event.pos())
            self.__skip_next_hide = not self.view().visualRect(index).contains(event.pos())
        return False



class MappingTable(QWidget):
    def __init__(self, column_labels=None):
        super(MappingTable, self).__init__()

        self.layout = QVBoxLayout(self)
        self.internal_map = {}  # This will mimic your HashMap<PowerDistributionType, f32>

        self.table_widget = QTableWidget()
        self.layout.addWidget(self.table_widget)

        if column_labels:
            self.configure_columns(column_labels)

    def configure_columns(self, column_labels):
        self.table_widget.setColumnCount(len(column_labels))
        self.table_widget.setHorizontalHeaderLabels(column_labels)

    def add_row(self, *widgets):
        row_count = self.table_widget.rowCount()
        self.table_widget.insertRow(row_count)

        for i, widget in enumerate(widgets):
            self.table_widget.setCellWidget(row_count, i, widget)
            widget_index = f"{row_count}_{i}"
            if isinstance(widget, ClickableComboBox):
                widget.currentIndexChanged.connect(lambda idx, widx=widget_index: self.update_internal_map(widx))
            elif isinstance(widget, QtWidgets.QDoubleSpinBox):
                widget.valueChanged.connect(lambda val, widx=widget_index: self.update_internal_map(widx))

    def update_internal_map(self, widget_index):
        row, col = map(int, widget_index.split("_"))
        widget = self.table_widget.cellWidget(row, col)

        if not row in self.internal_map:
            self.internal_map[row] = {}
        
        if isinstance(widget, ClickableComboBox):
            self.internal_map[row][f"combo_{col}"] = widget.currentText()
        elif isinstance(widget, QtWidgets.QDoubleSpinBox):
            self.internal_map[row][f"spin_{col}"] = widget.value()