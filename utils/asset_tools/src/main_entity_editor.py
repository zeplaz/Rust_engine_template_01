import PySide6 as py6
from PySide6.QtUiTools import QUiLoader
from .subsystems import *
from .style_sheet import set_dark_theme

from .pages.buildings_pages import BuildingsUI, PowerUI
from .pages.vehical_pages import VehicalUI
from .asset_config import SEGMENT_MEMBERSHIP,MASTER_ENTITY_FILTER_ROLES

import os 
import sys

loader = QUiLoader()
my_dir = os.path.dirname(os.path.abspath(__file__))

# capacity, mass, max_speed, military_civilian, texture_path. name

game_entities_dir = "/data/game_entities/"
asset_textures_dir = "/data/textures/"


# is_building, 





FILE_FILTERS = ["JSON (*.json)","PNG (*.png)","config (*.cfg)","data (*.dat)"]


class MainWindow(py6.QtWidgets.QMainWindow):
    def __init__(self):
        super(MainWindow, self).__init__()
        self.ui = loader.load(os.path.join(my_dir,"asset_tool_.ui"), None) 
        
        self.asset_classes_folder = []
        self.textures_folder = {}

        self.cbox_entity_filter_map = {}
        self.fraction_map = {}

        self.setupUi()
        
        self.ui.show()   
    
    def setupUi(self):
        
        display_resource_mechanism = py6.QtWidgets
        texture_properties = py6.QtWidgets.QComboBox()

        self.ui.add_asset_class_btn.clicked.connect(self.add_asset_class)
        
        self.setup_toolbox()

        doc_widget_container = py6.QtWidgets.QWidget()
        v_layout = py6.QtWidgets.QVBoxLayout(doc_widget_container)
        v_layout.setSpacing(1)
        v_layout.setContentsMargins(1,2,1,2)
        v_layout.setAlignment(py6.QtCore.Qt.AlignTop)
        
        label_EF = py6.QtWidgets.QLabel("Entity Filters")
        v_layout.addWidget(label_EF)
        v_layout.addWidget(self.setup_main_config_widget())

        label_Fraction = py6.QtWidgets.QLabel("Fraction")
        v_layout.addWidget(label_Fraction)
        v_layout.addWidget(self.setup_fraction_widget())
        doc_widget_container.setLayout(v_layout)

        self.ui.dockWidget.setWidget(doc_widget_container)
        
        self.on_building_checked(self.cbox_entity_filter_map["Building"].isChecked())
        self.on_vehicle_checked(self.cbox_entity_filter_map["Vehicle"].isChecked())
        self.on_power_checked(self.cbox_entity_filter_map["Power"].isChecked())
        self.refresh_tabs()

    def setup_toolbox(self):
        
        for i in range(self.ui.tool_box.count()):
            self.ui.tool_box.removeItem(i)
        # self.ui.tool_box.addItem(self.mil_civ_widget, "Military/Civilian")

        # Add widgets to the tool box
        self.building_ui_index = BuildingsUI.create_ui(self.ui.tool_box)
        self.power_ui_index = PowerUI.create_ui(self.ui.tool_box)
        self.index_vehc_option = self.setup_vehicle_config_page(self.ui.tool_box)
         # Connect the stateChanged signals

    def on_building_checked(self, state):
        self.cbox_entity_filter_map["Vehicle"].setEnabled(not state)
        self.cbox_entity_filter_map["Transportable"].setEnabled(not state)
        self.ui.tool_box.setItemEnabled(self.building_ui_index, state)

    def on_vehicle_checked(self, state):
        self.cbox_entity_filter_map["Building"].setEnabled(not state)
        self.cbox_entity_filter_map["Power"].setEnabled(not state)
        #self.vehical_options_widget.setEnabled(state)
        self.ui.tool_box.setItemEnabled(self.index_vehc_option,state)
    
    def on_power_checked(self, state):
        self.cbox_entity_filter_map["Vehicle"].setEnabled(not state)
        self.cbox_entity_filter_map["Scenery"].setEnabled(not state)

        self.ui.tool_box.setItemEnabled(self.power_ui_index, state)

    def setup_main_config_widget(self):
         # main config options
        self.cbox_entity_filter_map = {}

        master_config_widget = py6.QtWidgets.QWidget()
        mc_layout = py6.QtWidgets.QHBoxLayout(master_config_widget)
        

        mc_layout.setContentsMargins(0,0,0,0)
        mc_layout.setSpacing(0)
        mc_layout.setAlignment(py6.QtCore.Qt.AlignTop)    

        for master_config in MASTER_ENTITY_FILTER_ROLES:
            self.cbox_entity_filter_map[master_config] = py6.QtWidgets.QCheckBox(master_config)
            self.cbox_entity_filter_map[master_config].stateChanged.connect(self.refresh_tabs)
            mc_layout.addWidget(self.cbox_entity_filter_map[master_config])
        master_config_widget.setLayout(mc_layout)

        self.cbox_entity_filter_map["Building"].stateChanged.connect(self.on_building_checked)
        self.cbox_entity_filter_map["Vehicle"].stateChanged.connect(self.on_vehicle_checked)
        self.cbox_entity_filter_map["Power"].stateChanged.connect(self.on_power_checked)
        return master_config_widget
        
    def setup_fraction_widget(self):

        # mil_civ radio fractions/SEGMENT_MEMBERSHIP options
        self.fraction_map = {}

        frac_civ_widget = py6.QtWidgets.QWidget()
        frac_layout = py6.QtWidgets.QHBoxLayout(frac_civ_widget)
        frac_layout.setContentsMargins(0,0,0,0)
        frac_layout.setSpacing(0)

        frac_btn_group = py6.QtWidgets.QButtonGroup()

        for fraction in SEGMENT_MEMBERSHIP:
            self.fraction_map[fraction] = py6.QtWidgets.QRadioButton(f"{fraction}")
            frac_btn_group.addButton(self.fraction_map[fraction])
            frac_layout.addWidget(self.fraction_map[fraction])
         
        frac_civ_widget.setLayout(frac_layout)

        return frac_civ_widget
        #self.ui.tool_box.addItem(master_config_tool_box_widget,"Main Config Options")

    def setup_vehicle_config_page(self, tool_box_widget):
        # vehicle config options

        return VehicalUI.create_ui(tool_box_widget)
      
    
    def add_asset_class(self):

        folder_path = py6.QtWidgets.QFileDialog.getExistingDirectory(self, "Add asset class")
        if folder_path:
            if os.path.isdir(folder_path):

               # Get all subfolders
               self.asset_classes_folder = [os.path.join(folder_path, f) for f in os.listdir(folder_path) if os.path.isdir(os.path.join(folder_path, f))]

               # Refresh the tabs
               self.refresh_tabs()
        """
        add_diolog = py6.QtWidgets.QFileDialog()
        layout = py6.QtWidgets.QHBoxLayout(add_diolog)
        layout.addWidget(py6.QtWidgets.QLabel("Add asset class"))

        
        add_asset_class_btn = py6.QtWidgets.QPushButton("Add asset class")
        add_asset_class_btn.clicked.connect(lambda: self.get_folder(self.asset_classes_folder))

        layout.addWidget(add_asset_class_btn)
        add_diolog.setLayout(layout)
        add_diolog.exec()
        """
    def refresh_tabs(self):

            # Get the list of current tab names
        current_tab_names = [self.ui.tab_assets.tabText(i) for i in range(self.ui.tab_assets.count())]

        # Remove tabs that are no longer in the self.asset_classes_folder list
        for tab_index in range(self.ui.tab_assets.count() - 1, -1, -1):
            tab_name = self.ui.tab_assets.tabText(tab_index)
            if tab_name not in [os.path.basename(folder_path) for folder_path in self.asset_classes_folder]:
                self.ui.tab_assets.removeTab(tab_index)


        if len(self.asset_classes_folder) != 0: 
            for folder in self.asset_classes_folder:
                for root, dirs, files in os.walk(folder):
                    tab_name = os.path.basename(root)
                    if tab_name not in current_tab_names:
                        asset_list = py6.QtWidgets.QListWidget()
        
                        for file in files:
                            asset_list.addItem(py6.QListWidgetItem(file))
                        
                        self.ui.tab_assets.addTab(asset_list,tab_name)


    @staticmethod
    def get_folder(self):  
       
        caption = "Select folder"
        init_dir = ""
        dialog = py6.QtWidgets.QFileDialog()
        dialog.setFileMode(py6.QtWidgets.QFileDialog.Directory)
        dialog.setOption(py6.QtWidgets.QFileDialog.ShowDirsOnly, True)
        dialog.setWindowTitle(caption)
        dialog.setDirectory(init_dir)

        ok = dialog.exec()
        if ok == py6.QFileDialog.Accepted:
           folder_path = dialog.selectedFiles()[0]
           if os.path.isdir(folder_path):

               # Get all subfolders
               self.asset_classes_folder = [os.path.join(folder_path, f) for f in os.listdir(folder_path) if os.path.isdir(os.path.join(folder_path, f))]

               # Refresh the tabs
               self.refresh_tabs()

        print("Result: ", ok,dialog.selectedFiles())
        

    def show_vehicle_options(self, show):
        vehicle_options  = [self.vehical_options_widget]

        vehicle_options_page_index = 1  # Adjust this to match the index of the "Vehicle Options" page

    # Find the widget item representing the "Vehicle Options" page in the QToolBox
        item = self.ui.tool_box.item(vehicle_options_page_index)

    # Enable or disable the entire page (widget) based on the 'show' parameter
        item.setEnabled(show)

        #for option in vehicle_options:
         
         #  self.ui.tool_box.findChild(py6.QtWidgets.QWidget, option.objectName()).setEnabled(show)
       
        #for i in range(3):
         #   self.ui.tool_box.setItemEnabled(i, show)

    def on_new_asset_class(self, asset):
        pass 
            #if asset is in 

    def display_asset_view(self):
        if self.texture_path is not None:
            texture_view = self.ui.texture_viewer
            texture_view.setScene(py6.QtWidgets.QGraphicsScene())
            texture_view.setTransformationAnchor(py6.QtWidgets.QGraphicsView.AnchorUnderMouse)
            texture_view.setHorizontalScrollBarPolicy(py6.QtCore.Qt.ScrollBarAlwaysOff)
            texture_view.setVerticalScrollBarPolicy(py6.QtCore.Qt.ScrollBarAlwaysOff)
            texture_view.setRenderHint(py6.QtGui.QPainter.Antialiasing)
            texture_view.setRenderHint(py6.QtGui.QPainter.SmoothPixmapTransform)
            texture_view.setRenderHint(py6.QtGui.QPainter.TextAntialiasing)
            texture_view.setRenderHint(py6.QtGui.QPainter.SmoothPixmapTransform)


        
    
    def add_textures(self):
        total_index = py6.QtWidgets.QSpinBox()
        add_texture_path_btn = py6.QtWidgets.QPushButton("Add tileMaps path")
        layout = py6.QtWidgets.QVBoxLayout()
        layout.addWidget(add_texture_path_btn)
        layout.addWidget(total_index)
        add_texture_path_btn.clicked.connect(self.get_folder,self.texture_paths)
        



def main():
    app = py6.QtWidgets.QApplication(sys.argv)
    set_dark_theme(app)
    window = MainWindow()
    sys.exit(app.exec())
    
    
