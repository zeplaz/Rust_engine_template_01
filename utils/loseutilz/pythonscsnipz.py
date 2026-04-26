# -*- coding: utf-8 -*-



`HierarchyView.instanceCreatedSignal()`to extend the menu any time a HierarchyView is created:
import functools

import SceneUI

def __printSelection( selection ) :

    print( selection.paths() )

def __customMenu( column, pathListing, menuDefinition ) :

    selection = pathListing.getSelection()

    menuDefinition.append(
        "Print Path{}".format( "" if selection.size() == 1 else "s" ),
        {
            "command" : functools.partial( __printSelection, selection ),
            "active" : not selection.isEmpty()
        }
    )

def __hierarchyViewCreated( hierarchyView ) :

    hierarchyView.sceneListing().columnContextMenuSignal().connect( __customMenu, scoped = False )

SceneUI.HierarchyView.instanceCreatedSignal().connect( __hierarchyViewCreated, scoped = False 