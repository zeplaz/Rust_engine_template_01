from PySide6 import QtCore, QtWidgets, QtGui
from ..subsystems import append_widget, create_widget_with_label
class VehicalUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.spinbox_veh_capacity = QtWidgets.QSpinBox()
        self.spindublebox_mass = QtWidgets.QDoubleSpinBox()
        self.spindublebox_mass.setSingleStep(0.03)
        self.spinbox_max_speed = QtWidgets.QSpinBox()

        vehical_options_widget = QtWidgets.QWidget()
        layout  = QtWidgets.QVBoxLayout(vehical_options_widget)
        layout.setSpacing(0)  # Set the spacing between items to zero
        layout.setContentsMargins(5, 0, 5, 0)

        layout.addWidget(create_widget_with_label(self.spindublebox_mass, 'Mass'))
        layout.addWidget(create_widget_with_label(self.spinbox_veh_capacity, 'Vehicle Capacity'))
        layout.addWidget(create_widget_with_label(self.spinbox_max_speed, 'Max Speed'))
        layout.addStretch(1) 
    
    def create_ui(parent):
        widget = VehicalUI()
        return append_widget(parent, widget, "Vehicle, Options")
