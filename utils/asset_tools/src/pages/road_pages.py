class RoadUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        widget = QtWidgets.QWidget(parent)
        layout_V1 = QtWidgets.QVBoxLayout(widget)

        layout_H1 = QtWidgets.QHBoxLayout()
        layout_V1.addLayout(layout_H1)

        parent.lanes = QtWidgets.QSpinBox(parent)
        parent.lanes.setMinimum(1)
        parent.lanes.setMaximum(10)
        parent.surface = QtWidgets.ComboBox(parent)
        parent.surface.addItems(SURFACE_TYPES)

    def create_road_ui(parent):
        widget = RoadUI()
        append_widget(parent, widget, "Road Options")