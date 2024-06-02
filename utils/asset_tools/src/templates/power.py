from ..subsystems import append_widget, create_widget_with_label, ClickableComboBox, ArrowHideableSpinBox, populate_from_config, MappingTable
from ..config_paths import VOLTAGE_TUPLE,REDEX_INDEX, FILE_PATH_INDEX

from PySide6 import QtWidgets, QtCore, QtGui
# Power
POWER_DISTRIBUTION_TYPES = [ "Three Phase Heavy Industrial","Three Phase Medium Industrial","One Phase Light Industrial",
                             "Three Phase Residential","One Phase Residential", "Three Phase Long Distance", "One phase Long Distance", "Mixed"]



def create_and_bind_qt_qt_widget(widget_type, parent_widget = None, name = None,predfined= None, **kwargs):
    if widget_type == "QComboBox":
        widget = QtWidgets.QComboBox(parent_widget, **kwargs)
        if predfined:   
             widget.addItems(predfined)
        return create_widget_with_label(widget,name)
    elif widget_type == "QLineEdit":
        return  create_widget_with_label(QtWidgets.QLineEdit(parent_widget, **kwargs),name)
    elif widget_type == "QSpinBox":
        return create_widget_with_label(QtWidgets.QSpinBox(parent_widget, **kwargs),name)
    elif widget_type == "QDoubleSpinBox":
        return create_widget_with_label(QtWidgets.QDoubleSpinBox(parent_widget, **kwargs),name)
    elif widget_type == "QCheckBox":
        if predfined:   
            widget = QtWidgets.QCheckBox(parent_widget, predfined)        
        return create_widget_with_label(widget,name)
    elif widget_type == "ClickableComboBox":
        return create_widget_with_label(ClickableComboBox(parent_widget, **kwargs),name)

def bind_qt_widget_to_predefined_items(widget,predfined):
    widget.addItems(predfined)


eletrical_component_config = {
    "radius": ("QDoubleSpinBox", 0.25),
    "capacity": ("QDoubleSpinBox", 0.1),
    "max_temperature":("QDoubleSpinBox",0.3),
    "base_load":("QDoubleSpinBox",0.05),
}

# Define common configurations
common_config = {
    
    # ... other common key-values
}


# Create specialized configurations by unpacking the common one and updating only what's different
powerplant_config = {**eletrical_compoent_config, **{
    # ... powerplant-specific key-values
    "plant_type": "QComboBox"

}}

transformer_config = {**eletrical_compoent_config, **{
    #... trasfomer-specific key-values
    input_voltage:
}

substation_config = {**eletrical_compoent_config, **{
    #... substaion-specific key-values
    "max_transfer":("QDoubleSpinBox",0.01),
    "output_voltages":("ClickableComboBox",populate_from_config(VOLTAGE_TUPLE)),
    "input_voltages":("ClickableComboBox", populate_from_config(VOLTAGE_TUPLE)),
    "power_distribution_types":("ClickableComboBox",POWER_DISTRIBUTION_TYPES),
}}

# Finally, build the main dictionary
POWER_SUBTYPES = {
    "PowerPlant": powerplant_config,
    "Transformer": transformer_config,
    "Substation": substation_config,
    "Switch": switch_config,
    "PowerLine": PowerLine_config,

    # ... and so on for other subtypes
}                                

               
                
                 "Transformer", }



class power_plant_entity_template(object):
    def __init__(self, name):


class power_entity_template(object):
    def __init__(self, name, subtype):
        self.name = name
        self.sub_type = subtype
        self.componets = {}

    def add_subtype_params(self):
        self.componets[]
        if self.sub_type == "PowerPlant":
            
            # electrical load, thermalcompoent

        elif self.sub_type == "Substation":
            pass                        
        elif self.sub_type == "Transformer":
            pass 
        elif self.sub_type == "Switch":
            pass
        elif self.sub_type == "PowerLine":
            pass 
            



