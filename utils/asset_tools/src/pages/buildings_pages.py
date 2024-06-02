from PySide6 import QtWidgets, QtGui, QtCore

from ..subsystems import append_widget, create_widget_with_label, ClickableComboBox, ArrowHideableSpinBox, populate_from_config

   

class BuildingsUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)

        self.layout_H1 = QtWidgets.QHBoxLayout(self)
        self.layout_H1.setContentsMargins(0,0,0,0)
        self.layout_H1.setSpacing(0)
        self.layout_H1.setAlignment(QtCore.Qt.AlignmentFlag.AlignTop)

        self.layout_V1 = QtWidgets.QVBoxLayout()
        self.layout_V1.setContentsMargins(0,0,0,0)
        self.layout_V1.setSpacing(0)
        self.layout_V1.setAlignment(QtCore.Qt.AlignmentFlag.AlignTop)

        self.layout_H1.addLayout(self.layout_V1)
       
        label = QtWidgets.QLabel("Building Type:")
        self.layout_V1.addWidget(label)

        self.cbox_type = QtWidgets.QComboBox(self)
        self.cbox_type.addItems(BUILDING_TYPES)
        
        self.layout_V1.addWidget(self.cbox_type)

        self.cbox_type.currentIndexChanged.connect(self.refresh_building_ui)
        
        self.setLayout(self.layout_H1)

        self.refresh_building_ui()
       

    def refresh_building_ui(self):
        
        if hasattr(self,"options_widget") and self.options_widget is not None:
            self.options_widget.deleteLater()

        self.options_widget = QtWidgets.QWidget(self)
        o_layout = QtWidgets.QHBoxLayout(self.options_widget)
        self.options_widget.setLayout(o_layout)
        self.layout_H1.addWidget(self.options_widget)

        if self.cbox_type.currentText() == "Residencey":
            self.current_tool_index  = ApartmentUI.create_ui(self.options_widget)
        elif self.cbox_type.currentText() == "Depanneur":
           pass 
        elif self.cbox_type.currentText() == "Warehouse":
           pass 
        elif self.cbox_type.currentText() == "Power":
           self.current_tool_index = PowerUI.create_ui(self.options_widget)
        elif self.cbox_type.currentText() == "Farm":
           pass 
        elif self.cbox_type.currentText() == "Reserch":
           pass 
        elif self.cbox_type.currentText() == "Rail":
            pass 
        
    def create_ui(self):
        widget = BuildingsUI()
        return append_widget(self, widget, "Building Options")

class ResidenceyUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.layout_V1 = QtWidgets.QVBoxLayout(self)
        
        self.cbox_type = QtWidgets.QComboBox(self)
        self.cbox_type.addItems(RESEDENCY_TYPES)
        self.layout_V1.addWidget(self.cbox_type)
        self.cbox_type.currentIndexChanged.connect(self.refresh_residencey_ui)
    
    def refresh_residencey_ui(self):
        if self.cbox_type.currentText() == "Appartments":
            self.current_tool_index = ApartmentUI.create_ui(self)
    
    def create_ui(self):
         widget = ResidenceyUI()
         return append_widget(self, widget, "Residencey Options")

class ApartmentUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)

        self.appartment_units_spinboxes = {}

        self.layout_V1 = QtWidgets.QVBoxLayout(self)
        
        self.cbox_app_type = QtWidgets.QComboBox(self)
        self.cbox_app_type.addItems(APARTMENT_TYPES)
        self.layout_V1.addWidget(self.cbox_app_type)

        self.add_appartment_units()
    
    @staticmethod
    def create_ui(parent):
        widget = ApartmentUI()
        return append_widget(parent, widget, "Appartments Options")

    def remove_additional_options(self):
        # This function will remove additional options like floors, units per floor, and tile size
        for attr in ['floors_label', 'floors_spinbox', 'units_per_floor_label', 'units_per_floor_spinbox',
                     'tile_width_label', 'tile_width_spinbox', 'tile_height_label', 'tile_height_spinbox']:
            if hasattr(self, attr):
                widget = getattr(self, attr)
                widget.deleteLater()
                delattr(self, attr)


    def set_unit_limit(self, max_units):
        for _, spinbox in self.appartment_units_spinboxes.items():
            spinbox.setMaximum(max_units)

    def add_floor_label_read_only(self, limit):
        if not hasattr(self, 'floors_label'):
            self.floors_label = QtWidgets.QLabel("Number of floors:")
            self.floors_spinbox = ArrowHideableSpinBox()
            self.floors_spinbox.toggleArrows(False)
            self.floors_spinbox.setReadOnly(True)
            self.floors_spinbox.setValue(limit)
            self.layout_V1.addWidget(self.floors_label)
            self.layout_V1.addWidget(self.floors_spinbox)

    def update_ui_based_on_type(self, index):
        self.remove_additional_options()

        app_type = self.cbox_app_type.itemText(index)
        if app_type == "Duplex":
            self.set_unit_limit(2)
            self.add_floor_label_read_only(2) 
        elif app_type == "Quadplex":
            self.set_unit_limit(4)
            self.add_floor_label_read_only(2)
        elif app_type == "HighRise":
            self.add_highrise_options()
            self.update_units_limit_for_highrise()
        elif app_type in ["Three Story Block", "Five Story Block"]:
            self.add_tile_size_options(app_type)
            if app_type == "Three Story Block":
                 self.add_floor_label_read_only(3)
            if app_type == "Five Story Block":
                self.add_floor_label_read_only(5)
        else:
            self.remove_additional_options()
    
    def update_units_limit_for_highrise(self):
        max_units = self.floors_spinbox.value() * self.units_per_floor_spinbox.value()
        for _, spinbox in self.appartment_units_spinboxes.items():
            spinbox.setMaximum(max_units)

    def add_appartment_units(self):
        self.cc_box_units_available = ClickableComboBox(self)
        self.cc_box_units_available.addItems(APARTMENT_UNIT_TYPES)
        self.layout_V1.addWidget(self.cc_box_units_available)

        self.appart_units_widiget = QtWidgets.QWidget(self)
        self.appart_units_widiget.setLayout(QtWidgets.QVBoxLayout())
        self.layout_V1.addWidget(self.appart_units_widiget)

        # Connect the update signal
        self.cc_box_units_available.model().dataChanged.connect(self.refresh_appartment_units_ui)
        """ 
                        parent.cc_box_units_available.model().dataChanged.connect(
                        lambda: refresh_appartment_units_ui(parent, parent.appart_units_widiget)
                        )
        """
        self.cbox_app_type.currentIndexChanged.connect(self.update_ui_based_on_type)
        self.update_ui_based_on_type(self.cbox_app_type.currentIndex())  # Initial UI configuration based on the default selection
        self.refresh_appartment_units_ui()  # Initial population of the UI
    
    def add_highrise_options(self):
        # If the options are already present, avoid re-adding them.
        
        if hasattr(self, 'floors_spinbox'):
            try:
                self.floors_spinbox.toggleArrows(True)
            except exeption as e:
                print("Error: floors_spinbox not init: {}".format(e))        
            return

        self.floors_label = QtWidgets.QLabel("Number of floors:")
        self.floors_spinbox = ArrowHideableSpinBox()
        
        self.units_per_floor_label = QtWidgets.QLabel("Units per floor:")
        self.units_per_floor_spinbox = QtWidgets.QSpinBox()

        self.floors_spinbox.valueChanged.connect(self.update_units_limit_for_highrise)
        self.units_per_floor_spinbox.valueChanged.connect(self.update_units_limit_for_highrise)

        self.layout_V1.addWidget(self.floors_label)
        self.layout_V1.addWidget(self.floors_spinbox)
        self.layout_V1.addWidget(self.units_per_floor_label)
        self.layout_V1.addWidget(self.units_per_floor_spinbox)

    def add_tile_size_options(self, app_type):
            # If the options are already present, avoid re-adding them.
            if hasattr(self, 'tile_width_spinbox'):
                return
            self.tile_width_label = QtWidgets.QLabel("Tile width:")
            self.tile_width_spinbox = QtWidgets.QSpinBox()
            self.tile_height_label = QtWidgets.QLabel("Tile height:")
            self.tile_height_spinbox = QtWidgets.QSpinBox()

            # Set limits based on the app_type
            if app_type == "Three Story Block":
                # You can adjust these limits as per your requirement
                self.tile_width_spinbox.setMaximum(10)
                self.tile_height_spinbox.setMaximum(5)
            elif app_type == "Five Story BlocK":
                self.tile_width_spinbox.setMaximum(3)
                self.tile_height_spinbox.setMaximum(4)

            self.layout_V1.addWidget(self.tile_width_label)
            self.layout_V1.addWidget(self.tile_width_spinbox)
            self.layout_V1.addWidget(self.tile_height_label)
            self.layout_V1.addWidget(self.tile_height_spinbox)    
    
    def refresh_appartment_units_ui(self):
        # Clear existing widgets
        for i in reversed(range(self.appart_units_widiget.layout().count())):
            widget = self.appart_units_widiget.layout().itemAt(i).widget()
            if widget is not None:
                widget.deleteLater()
        
        # Re-add based on current state
        for app_unit in self.cc_box_units_available.currentData():
            label = QtWidgets.QLabel(app_unit)
            spin_box = QtWidgets.QSpinBox(self.appart_units_widiget)

            # Store the SpinBox in the dictionary for later access
            self.appartment_units_spinboxes[app_unit] = spin_box

            self.appart_units_widiget.layout().addWidget(label)
            self.appart_units_widiget.layout().addWidget(spin_box)


# BUILDING_TYPES= [ "Residencey","Depanneur" "Burocracy", "Feild Depo","Warehouse","Factory","Mine","Fuel","Power","Reserch","Farm","Rail"]



class DepanneursUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)

    @staticmethod
    def create_ui(parent):
        widget = DepanneursUI()
        return append_widget(parent, widget, "Depanneurs Options")


class BurocracysUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
    
    @staticmethod
    def create_ui(parent):
        widget = BurocracysUI()
        return append_widget(parent, widget, "Burocracys Options")

class FeildDeposUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
    
    @staticmethod
    def create_ui(parent):
        widget = FeildDeposUI()
        return append_widget(parent, widget, "Feild Depos Options")

class WarehousesUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
    
    @staticmethod
    def create_ui(parent):
        widget = WarehousesUI()
        return append_widget(parent, widget, "Warehouses Options")
        
class FactoriesUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
    
    @staticmethod
    def create_ui(parent):
        widget = FactoriesUI()
        return append_widget(parent, widget, "Factories Options")


class MinesUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
    
    @staticmethod
    def create_ui(parent):
        widget = MinesUI()
        return append_widget(parent, widget, "Mines Options")


class FuelUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
    
    @staticmethod
    def create_ui(parent):
        widget = FuelStationsUI()
        return append_widget(parent, widget, "Fuel Options")



class ReserchUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
    
    @staticmethod
    def create_ui(parent):
        widget = ReserchUI()
        return append_widget(parent, widget, "Reserch Options")

class FarmsUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
    
    @staticmethod
    def create_ui(parent):
        widget = FarmsUI()
        return append_widget(parent, widget, "Farms Options")

class RailUI(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
    
    @staticmethod
    def create_ui(parent):
        widget = RailUI()
        return append_widget(parent, widget, "Rail Options")
