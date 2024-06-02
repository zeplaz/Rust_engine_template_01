

from PySide6 import QtWidgets, QtGui, QtCore

from ..subsystems import append_widget, create_widget_with_label, ClickableComboBox, ArrowHideableSpinBox, populate_from_config
from ..config_paths import VOLTAGE_TUPLE,REDEX_INDEX, FILE_PATH_INDEX
from ..templates.power import POWER_DISTRIBUTION_TYPES, POWER_SUBTYPES


class PowerUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)

        power_config_widget = QtWidgets.QWidget()
        
        laout_H2 = QtWidgets.QVBoxLayout(power_config_widget)
        laout_H2.setSpacing(0)  # Set the spacing between items to zero
        laout_H2.setAlignment(QtCore.Qt.AlignmentFlag.AlignTop)
        laout_H2.setContentsMargins(5, 0, 5, 0)
        
        power_config_layout = QtWidgets.QVBoxLayout()
        power_config_layout.setSpacing(0)  # Set the spacing between items to zero
        power_config_layout.setAlignment(QtCore.Qt.AlignmentFlag.AlignTop)
        power_config_layout.setContentsMargins(5, 0, 5, 0)
        p_button_group = QtWidgets.QButtonGroup(self)

        self.radio_subtypes = {}
        
        self.ui_setup()

        self.setLayout(laout_H2)
        if isinstance(self.parent(), QtWidgets.QToolBox):
            self.parent().currentChanged.connect(self.handle_parent_state_change)
        #self.parent().enabledChanged.connect(self.handle_parent_state_change)
        #self.parent().visibleChanged.connect(self.handle_parent_state_change)

    def ui_setup(self):
        for subtype in POWER_SUBTYPES:
            button = QtWidgets.QRadioButton(subtype)
            button.stateChanged.connect(self.update_ui)
            self.radio_subtypes[subtype] = button
            p_button_group.addButton(button)
            laout_H2.addWidget(button)

    def update_ui(self):

        if self.check_substation.isChecked():
            self.layout().addWidget(self.substation_widget())
        else:
            self.layout().removeWidget(self.substation_widget())
            self.substation_widget().setParent(None)
    
    def add_ditro_types(self):
        self.distro_combobox = ClickableComboBox()
        for distrotype in POWER_DISTRIBUTION_TYPES:
            self.distro_combobox.addItem(distrotype)
        self.distro_combobox.currentIndexChanged.connect(self.update_ui)

            
    @staticmethod
    def create_ui(parent):
        widget = PowerUI(parent)
        return append_widget(parent, widget, "Power Config")

    @QtCore.Slot(int)
    def handle_parent_state_change(self):
        if index == self.parent().indexOf(self):
            self.show()
        else:
            self.hide()

    def add_thermal_compoent(self):
        widget = QtWidgets.QWidget()
        layoutv1 = QtWidgets.QVBoxLayout()
        layouth1 = QtWidgets.QHBoxLayout()
        layoutv1.setContentsMargins(5, 0,5, 0)
        layouth1.setContentsMargins(5, 0, 5, 0)
        
        label = QtWidgets.QLabel("thermal")
        font = label.font()
        font.setBold(True)                      # Make the font bold
        font.setPointSize(font.pointSize() + 2)  # Increase the font size slightly
        label.setFont(font)
        layoutv1.addWidget(label,alignment=QtCore.Qt.AlignmentFlag.AlignLeft)

        self.current_temperature = QtWidgets.QDoubleSpinBox()
        self.current_temperature.setSingleStep(0.01)
        self.max_temperature = QtWidgets.QDoubleSpinBox()
        self.max_temperature.setSingleStep(0.01)

        layouth1.addWidget(create_widget_with_label(self.current_temperature, "Current Temperature"))
        layouth1.addWidget(create_widget_with_label(self.max_temperature, "Max Temperature"))


    def add_eletrical_load_compoent(self):
        widget = QtWidgets.QWidget()
        layoutv1 = QtWidgets.QVBoxLayout()
        layouth1 = QtWidgets.QHBoxLayout()
        layoutv1.setContentsMargins(5, 0,5, 0)
        layouth1.setContentsMargins(5, 0, 5, 0)
        
        label = QtWidgets.QLabel("eletrical load")
        font = label.font()
        font.setBold(True)                      # Make the font bold
        font.setPointSize(font.pointSize() + 2)  # Increase the font size slightly
        label.setFont(font)
        layoutv1.addWidget(label,alignment=QtCore.Qt.AlignmentFlag.AlignLeft)

        self.base_load = QtWidgets.QDoubleSpinBox()
        self.base_load.setSingleStep(0.01)
        self.max_transfer = QtWidgets.QDoubleSpinBox()
        self.max_transfer.setSingleStep(0.01)
        
        layouth1.addWidget(create_widget_with_label(self.base_load, "Base Load"))
        layouth1.addWidget(create_widget_with_label(self.max_transfer, "Max Transfer"))


    def trasformer_widget(self):
        widget = QtWidgets.QWidget()
        layoutv1 = QtWidgets.QVBoxLayout()
        layouth1 = QtWidgets.QHBoxLayout()
        layoutv1.setContentsMargins(5, 0,5, 0)
        layouth1.setContentsMargins(5, 0, 5, 0)
        
        label = QtWidgets.QLabel("transformer")
        font = label.font()
        font.setPointSize(font.pointSize() + 2)  # Increase the font size slightly
        label.setFont(font)
        layoutv1.addWidget(label,alignment=QtCore.Qt.AlignmentFlag.AlignLeft)


    def substation_widget(self):
        widget = QtWidgets.QWidget()
        layoutv1 = QtWidgets.QVBoxLayout()
        layouth1 = QtWidgets.QHBoxLayout()
        layoutv1.setContentsMargins(5, 0,5, 0)
        layouth1.setContentsMargins(5, 0, 5, 0)

        label = QtWidgets.QLabel("substation")
        font = label.font()
        font.setBold(True)                      # Make the font bold
        font.setPointSize(font.pointSize() + 2)  # Increase the font size slightly
        label.setFont(font)
        layoutv1.addWidget(label,alignment=QtCore.Qt.AlignmentFlag.AlignLeft)

        self.substation_spind_capacity = QtWidgets.QDoubleSpinBox()
        self.substation_spind_capacity.setSingleStep(0.01)
        self.substation_spind_radius = QtWidgets.QDoubleSpinBox()
        self.substation_spind_radius.setSingleStep(0.01)
        self.substation_spind_max_transfer = QtWidgets.QDoubleSpinBox()
        self.substation_spind_max_transfer.setSingleStep(0.01)
        self.substation_comb_output_voltages = ClickableComboBox()
        self.substation_comb_input_voltages = ClickableComboBox()

        populate_from_config(VOLTAGE_TOUPLE[FILE_PATH_INDEX],VOLTAGE_TOUPLE[REDEX_INDEX],self.substation_comb_output_voltages)
        populate_from_config(VOLTAGE_TOUPLE[FILE_PATH_INDEX],VOLTAGE_TOUPLE[REDEX_INDEX],self.substation_comb_input_voltages)

        self.substation_spind_max_temperature = QtWidgets.QDoubleSpinBox()

        layouth1.addWidget(create_widget_with_label(self.substation_spind_capacity,'Capacity'))
        layouth1.addWidget(create_widget_with_label(self.substation_spind_radius,'Radius'))
        layouth1.addWidget(create_widget_with_label(self.substation_spind_max_transfer,'Max Transfer'))
        layouth1.addWidget(create_widget_with_label(self.substation_comb_output_voltages,'Output Voltage'))
        layouth1.addWidget(create_widget_with_label(self.substation_comb_input_voltages,'Input Voltage'))
        layouth1.addWidget(create_widget_with_label(self.substation_spind_max_temperature,'Max Temperature'))
        
        layoutv1.addLayout(layouth1)
        widget.setLayout(layoutv1)

        return widget