"""Prop / corner module — greybox boxes; prop_kind selects L-corner vs vent/ac."""

from __future__ import annotations

import bpy


def _add_box(name: str, w: float, h: float, d: float, loc: tuple[float, float, float]) -> bpy.types.Object:
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = (w, h, d)
    bpy.ops.object.transform_apply(scale=True)
    return obj


def _build_l_corner(w: float, h: float, d: float, leg: float) -> bpy.types.Object:
    leg_w = w * leg
    leg_d = d * leg
    a = _add_box("corner_leg_a", leg_w, h, d, (leg_w * 0.5 - w * 0.5, h * 0.5, 0.0))
    b = _add_box("corner_leg_b", w, h, leg_d, (0.0, h * 0.5, leg_d * 0.5 - d * 0.5))
    bpy.ops.object.select_all(action="DESELECT")
    a.select_set(True)
    b.select_set(True)
    bpy.context.view_layer.objects.active = a
    bpy.ops.object.join()
    return bpy.context.active_object


def build(params: dict) -> bpy.types.Object:
    w = float(params.get("width_m", 2.0))
    h = float(params.get("height_m", 2.0))
    d = float(params.get("depth_m", 2.0))
    kind = str(params.get("prop_kind", "box"))

    if kind in ("l_corner", "corner", "corner_l"):
        obj = _build_l_corner(w, h, d, leg=float(params.get("leg_ratio", 0.45)))
    elif kind in ("chimney", "prop_chimney"):
        base = _add_box("chimney_base", w * 0.85, h * 0.72, d * 0.85, (0.0, h * 0.36, 0.0))
        stack = _add_box("chimney_stack", w * 0.55, h * 0.55, d * 0.55, (0.0, h * 0.78, 0.0))
        cap = _add_box("chimney_cap", w * 0.95, max(h * 0.12, 0.15), d * 0.95, (0.0, h * 0.96, 0.0))
        bpy.ops.object.select_all(action="DESELECT")
        base.select_set(True)
        stack.select_set(True)
        cap.select_set(True)
        bpy.context.view_layer.objects.active = base
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("vent", "ac"):
        base_h = h * 0.65
        cap_h = h - base_h
        base = _add_box("prop_base", w, base_h, d, (0.0, base_h * 0.5, 0.0))
        cap = _add_box(
            "prop_cap",
            w * 0.7,
            max(cap_h, 0.05),
            d * 0.7,
            (0.0, base_h + cap_h * 0.5, 0.0),
        )
        bpy.ops.object.select_all(action="DESELECT")
        base.select_set(True)
        cap.select_set(True)
        bpy.context.view_layer.objects.active = base
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("transformer", "prop_transformer"):
        radius = min(d, h) * 0.35
        length = w * 0.55
        bpy.ops.mesh.primitive_cylinder_add(
            radius=radius,
            depth=length,
            location=(0.0, radius + 0.05, 0.0),
        )
        tank = bpy.context.active_object
        tank.name = "transformer_tank"
        tank.rotation_euler[1] = 1.5707963
        bpy.ops.object.transform_apply(rotation=True)
        bushing_r = max(radius * 0.22, 0.08)
        bushing_h = max(h * 0.28, 0.12)
        offsets = (-length * 0.28, 0.0, length * 0.28)
        parts: list[bpy.types.Object] = [tank]
        for idx, ox in enumerate(offsets):
            bpy.ops.mesh.primitive_cylinder_add(
                radius=bushing_r,
                depth=bushing_h,
                location=(ox, radius * 2.0 + bushing_h * 0.5, 0.0),
            )
            bushing = bpy.context.active_object
            bushing.name = f"transformer_bushing_{idx}"
            parts.append(bushing)
        pad = _add_box("transformer_pad", w, max(h * 0.08, 0.08), d, (0.0, max(h * 0.04, 0.04), 0.0))
        parts.append(pad)
        bpy.ops.object.select_all(action="DESELECT")
        for part in parts:
            part.select_set(True)
        bpy.context.view_layer.objects.active = tank
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("bus_bay",):
        base = _add_box("bus_base", w, h * 0.72, d, (0.0, h * 0.36, 0.0))
        beam = _add_box("bus_beam", w * 0.9, max(h * 0.12, 0.15), d * 0.25, (0.0, h * 0.86, 0.0))
        bpy.ops.object.select_all(action="DESELECT")
        base.select_set(True)
        beam.select_set(True)
        bpy.context.view_layer.objects.active = base
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("breaker",):
        housing = _add_box("breaker_housing", w * 0.9, h * 0.78, d * 0.9, (0.0, h * 0.39, 0.0))
        stack = _add_box("breaker_stack", w * 0.45, h * 0.35, d * 0.45, (0.0, h * 0.9, 0.0))
        bpy.ops.object.select_all(action="DESELECT")
        housing.select_set(True)
        stack.select_set(True)
        bpy.context.view_layer.objects.active = housing
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("shack", "control_shack"):
        body = _add_box("shack_body", w, h * 0.88, d, (0.0, h * 0.44, 0.0))
        door = _add_box("shack_door", w * 0.35, h * 0.55, max(d * 0.06, 0.08), (0.0, h * 0.3, d * 0.47))
        bpy.ops.object.select_all(action="DESELECT")
        body.select_set(True)
        door.select_set(True)
        bpy.context.view_layer.objects.active = body
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("fence", "fence_chainlink"):
        panel = _add_box("fence_panel", w, h, max(d, 0.08), (0.0, h * 0.5, 0.0))
        post_l = _add_box("fence_post_l", max(w * 0.06, 0.08), h * 1.02, max(d * 0.5, 0.08), (-w * 0.45, h * 0.51, 0.0))
        post_r = _add_box("fence_post_r", max(w * 0.06, 0.08), h * 1.02, max(d * 0.5, 0.08), (w * 0.45, h * 0.51, 0.0))
        bpy.ops.object.select_all(action="DESELECT")
        panel.select_set(True)
        post_l.select_set(True)
        post_r.select_set(True)
        bpy.context.view_layer.objects.active = panel
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("gravel_pad",):
        obj = _add_box("gravel_pad", w, max(h, 0.08), d, (0.0, max(h, 0.08) * 0.5, 0.0))
    elif kind in ("warning_sign",):
        post = _add_box("sign_post", max(w * 0.18, 0.08), h * 0.72, max(d * 0.5, 0.08), (0.0, h * 0.36, 0.0))
        board = _add_box("sign_board", w, h * 0.42, max(d, 0.06), (0.0, h * 0.82, 0.0))
        bpy.ops.object.select_all(action="DESELECT")
        post.select_set(True)
        board.select_set(True)
        bpy.context.view_layer.objects.active = post
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("yard_kit",):
        slab = _add_box("yard_slab", w, max(h * 0.08, 0.12), d, (0.0, max(h * 0.04, 0.06), 0.0))
        frame = _add_box("yard_frame", w * 0.96, h * 0.55, d * 0.96, (0.0, h * 0.35, 0.0))
        bpy.ops.object.select_all(action="DESELECT")
        slab.select_set(True)
        frame.select_set(True)
        bpy.context.view_layer.objects.active = slab
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("containment_dome", "containment_dome_pwr"):
        base_h = h * 0.42
        dome_r = min(w, d) * 0.42
        base = _add_box("containment_base", w * 0.92, base_h, d * 0.92, (0.0, base_h * 0.5, 0.0))
        bpy.ops.mesh.primitive_uv_sphere_add(
            segments=24,
            ring_count=12,
            radius=dome_r,
            location=(0.0, base_h + dome_r * 0.82, 0.0),
        )
        dome = bpy.context.active_object
        dome.name = "containment_dome"
        dome.scale = (1.0, 0.72, 1.0)
        bpy.ops.object.transform_apply(scale=True)
        bpy.ops.object.select_all(action="DESELECT")
        base.select_set(True)
        dome.select_set(True)
        bpy.context.view_layer.objects.active = base
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("turbine_hall", "turbine_hall_1u"):
        hall_h = h * 0.72
        body = _add_box("turbine_hall_body", w, hall_h, d, (0.0, hall_h * 0.5, 0.0))
        ridge = _add_box("turbine_hall_ridge", w * 0.88, max(h * 0.12, 0.15), d * 0.35, (0.0, hall_h + max(h * 0.06, 0.08), 0.0))
        bpy.ops.object.select_all(action="DESELECT")
        body.select_set(True)
        ridge.select_set(True)
        bpy.context.view_layer.objects.active = body
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("cooling_tower", "cooling_tower_1u"):
        base_r = min(w, d) * 0.42
        top_r = base_r * 0.62
        mid_h = h * 0.78
        bpy.ops.mesh.primitive_cylinder_add(
            vertices=20,
            radius=base_r,
            depth=mid_h,
            location=(0.0, mid_h * 0.5, 0.0),
        )
        tower = bpy.context.active_object
        tower.name = "cooling_tower_body"
        bpy.ops.mesh.primitive_cone_add(
            vertices=20,
            radius1=top_r,
            radius2=top_r * 0.35,
            depth=max(h * 0.22, 0.4),
            location=(0.0, mid_h + max(h * 0.11, 0.2), 0.0),
        )
        cap = bpy.context.active_object
        cap.name = "cooling_tower_cap"
        bpy.ops.object.select_all(action="DESELECT")
        tower.select_set(True)
        cap.select_set(True)
        bpy.context.view_layer.objects.active = tower
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("diesel_gen_pad", "diesel_gen_pad_2x2"):
        pad_h = max(h * 0.12, 0.12)
        pad = _add_box("diesel_pad", w, pad_h, d, (0.0, pad_h * 0.5, 0.0))
        gen = _add_box("diesel_gen", w * 0.55, h * 0.55, d * 0.45, (0.0, pad_h + h * 0.28, 0.0))
        exhaust = _add_box("diesel_exhaust", w * 0.12, h * 0.35, d * 0.12, (w * 0.28, pad_h + h * 0.62, 0.0))
        bpy.ops.object.select_all(action="DESELECT")
        pad.select_set(True)
        gen.select_set(True)
        exhaust.select_set(True)
        bpy.context.view_layer.objects.active = pad
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("switchyard_edge", "switchyard_edge_1u"):
        base = _add_box("switchyard_base", w, h * 0.65, d, (0.0, h * 0.33, 0.0))
        bus = _add_box("switchyard_bus", w * 0.85, max(h * 0.1, 0.12), d * 0.18, (0.0, h * 0.82, 0.0))
        bpy.ops.object.select_all(action="DESELECT")
        base.select_set(True)
        bus.select_set(True)
        bpy.context.view_layer.objects.active = base
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("warning_sign_nuclear", "warning_sign_nuclear_1u"):
        post = _add_box("nuclear_sign_post", max(w * 0.16, 0.08), h * 0.68, max(d * 0.45, 0.08), (0.0, h * 0.34, 0.0))
        board = _add_box("nuclear_sign_board", w, h * 0.38, max(d, 0.06), (0.0, h * 0.8, 0.0))
        trefoil = _add_box("nuclear_trefoil", w * 0.28, h * 0.18, max(d * 0.5, 0.06), (0.0, h * 0.82, d * 0.12))
        bpy.ops.object.select_all(action="DESELECT")
        post.select_set(True)
        board.select_set(True)
        trefoil.select_set(True)
        bpy.context.view_layer.objects.active = post
        bpy.ops.object.join()
        obj = bpy.context.active_object
    elif kind in ("nuclear_yard_kit",):
        slab = _add_box("nuclear_yard_slab", w, max(h * 0.06, 0.1), d, (0.0, max(h * 0.03, 0.05), 0.0))
        berm = _add_box("nuclear_yard_berm", w * 0.94, h * 0.42, d * 0.94, (0.0, h * 0.28, 0.0))
        bpy.ops.object.select_all(action="DESELECT")
        slab.select_set(True)
        berm.select_set(True)
        bpy.context.view_layer.objects.active = slab
        bpy.ops.object.join()
        obj = bpy.context.active_object
    else:
        obj = _add_box("module_prop", w, h, d, (0.0, h * 0.5, 0.0))

    obj.name = params.get("name", "module_prop")
    return obj
