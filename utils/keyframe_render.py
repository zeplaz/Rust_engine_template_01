bl_info = {
    "name": "Keyframe Renderer",
    "blender": (3, 3, 6),
    "category": "Object",
}

import bpy
from bpy.types import Operator, Panel, PropertyGroup, UIList
from bpy.props import StringProperty, BoolProperty, CollectionProperty, PointerProperty

#bpy.utils.register_module(__name__)


def register():
   
 
    #UI
    bpy.utils.register_class(RENDER_PT_keyframes_panel)

    #ops
    bpy.utils.register_class(RENDER_OT_keyframes_to_images)

    # custom
    bpy.utils.register_class(keyframe_checkbox_group)

    
    #datatypes
    bpy.types.Scene.render_keyframes_output_name = StringProperty(name="File Name", default="keyframe")
    
    #bpy.types.Scene.prt_keyframe_checkboxes_group = bpy.props.PointerProperty(type=)
    bpy.types.Scene.keyframes_collection = bpy.props.CollectionProperty(type=keyframe_checkbox_group)

    bpy.app.handlers.depsgraph_update_post.append(update_keyframes_collection)
    bpy.types.Object.keyframes_updated = bpy.props.BoolProperty(default=False)
    bpy.types.Scene.prev_active_object = bpy.props.PointerProperty(type=bpy.types.Object)


def unregister():
    bpy.utils.unregister_class(keyframe_checkbox_group)
    #bpy.utils.unregister_class()

    bpy.utils.unregister_class(RENDER_PT_keyframes_panel)
    bpy.utils.unregister_class(RENDER_OT_keyframes_to_images)
    
    bpy.app.handlers.depsgraph_update_post.remove(update_keyframes_collection)

    
    del bpy.types.Scene.render_keyframes_output_name
    #del bpy.types.Scene.prt_keyframe_checkboxes_group
    del bpy.types.Scene.keyframes_collection
    del bpy.types.Scene.prev_active_object


class Keyframe:
    def __init__(self, name, value):
        self.name = name
        self.active = value
    
def update_keyframes_collection(depsgraph):
    context = bpy.context
    scene = context.scene
    obj = context.active_object
    if scene.prev_active_object != obj:
        scene.prev_active_object = obj

        # Clear the collection before updating
        if obj and obj.animation_data and obj.animation_data.action and obj.animation_data.action.fcurves:
            print("updating collections")
            scene.keyframes_collection.clear()
            keyframe_properties = get_key_frames_objects(scene, obj.animation_data.action.fcurves)
            add_checkboxes_to_scene_collection(scene, keyframe_properties)
            obj.keyframes_updated = True

def get_key_frames_objects(scene, fcurves):
    keyframe_properties = []
    frame_keyframes = {}  # Store keyframes per frame number

    for fcurve in fcurves:
        data_path = fcurve.data_path.split(".")[-1]
        for keyframe in fcurve.keyframe_points:
            frame = int(keyframe.co.x)

            if frame not in frame_keyframes:
                frame_keyframes[frame] = []

            frame_keyframes[frame].append(data_path)

    for frame, keyframes in frame_keyframes.items():
        keyframes_label = ', '.join(set(keyframes))
        print(keyframes_label)
        keyframe_properties.append(Keyframe(f"Frame {frame} ({keyframes_label})", False))

    return keyframe_properties
          
def add_checkboxes_to_scene_collection(scene, key_group):
    for kf in key_group:
        item = scene.keyframes_collection.add()
        item.keyframe_name = kf.name
        item.active = kf.active


class keyframe_checkbox_group(bpy.types.PropertyGroup):
    keyframe_name: bpy.props.StringProperty(name="keyframe_name")
    active: bpy.props.BoolProperty(name="active")


class RENDER_PT_keyframes_panel(Panel):
    bl_label = "Keyframes"
    bl_space_type = 'PROPERTIES'
    bl_region_type = 'WINDOW'
    bl_context = "output"
    bl_category = "Render"
    def draw(self, context):
            layout = self.layout
            scene = context.scene
            obj = context.active_object
           
             # Ensure there are animation data and fcurves in the scene
            if obj.animation_data:
                if obj.animation_data.action:
                    if obj.animation_data.action.fcurves:                        
                        for checkbox in scene.keyframes_collection:
                            name = checkbox.keyframe_name  
                            layout.prop(checkbox, "active", text=name)
           
                    else:
                         layout.label(text="No scene.animation_data.action.fcurves found")
                else:
                    layout.label(text="No atniamation_data.action found")

            else:
                layout.label(text="No object animation data found")

            layout.prop(context.scene, "render_keyframes_output_name", text="File Name")
            layout.operator("render.keyframes_to_images")


class RENDER_OT_keyframes_to_images(Operator):
    bl_idname = "render.keyframes_to_images"
    bl_label = "Render Keyframes to Images"

    def execute(self, context):
       
        file_path = bpy.path.abspath("//")
        base_filename = context.scene.render_keyframes_output_name

        scene = context.scene
        selected_frames = set()

        for checkbox in scene.keyframes_collection:
            if checkbox.active:
                frame_number = int(checkbox.keyframe_name.split("Frame ")[1].split(" ")[0])


                selected_frames.add(frame_number)

        for frame in selected_frames:
            scene.frame_set(frame)
            filepath = f"{file_path}{frame}_{base_filename}_"
            print("saved too:".format(filepath))
            bpy.context.scene.render.filepath = filepath
            bpy.ops.render.render(write_still=True)


        return {'FINISHED'}




if __name__ == "__main__":
    register()