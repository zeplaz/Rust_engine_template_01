"""One-off generator: writes assets/config/power/plant_definitions.json"""
import json
from pathlib import Path

STATUSES = [
    "Standby",
    "Operational",
    "Maintenance",
    "OutOfFuel",
    "StartingUp",
    "ShuttingDown",
    "Decommissioned",
    "ExternalShutdown",
    "ReducedCapacity",
    "OverCapacity",
    "EnvironmentalShutdown",
]

PLANTS = [
    ("Coal", "coal_ultra_supercritical_650mw_v1", "Ultra-supercritical coal, FGD + SCR"),
    ("Oil", "heavy_fuel_oil_steam_120mw_v1", "Residual-oil steam turbine + tank farm"),
    ("Gas", "cc_gtu_h_class_500mw_v1", "H-class combined-cycle block"),
    ("Biomass", "bubbling_fb_biomass_40mw_v1", "BFB / grate biomass boiler-steam"),
    ("Nuclear", "pwr_4loop_1100mw_v1", "Large four-loop PWR"),
    ("Hydro", "reaction_francis_medium_head_v1", "Francis reaction, medium-head scheme"),
    ("Solar", "utility_tracker_pv_150mw_ac_v1", "Single-axis tracker PV farm"),
    ("Wind", "class_III_turbine_80m_hub_3mw_v1", "Class III site IEC turbine repower"),
    ("Geothermal", "orc_binary_brine_45mw_v1", "ORC binary plant on brine loop"),
]

NAMEPLATE = {
    "Coal": 650.0,
    "Oil": 120.0,
    "Gas": 500.0,
    "Biomass": 40.0,
    "Nuclear": 1100.0,
    "Hydro": 210.0,
    "Solar": 150.0,
    "Wind": 3.0,
    "Geothermal": 45.0,
}
EFF = {
    "Coal": 0.43,
    "Oil": 0.38,
    "Gas": 0.58,
    "Biomass": 0.32,
    "Nuclear": 0.33,
    "Hydro": 0.90,
    "Solar": 0.22,
    "Wind": 0.38,
    "Geothermal": 0.12,
}
FUEL = {
    "Coal": 18.0,
    "Oil": 48.0,
    "Gas": 32.0,
    "Biomass": 22.0,
    "Nuclear": 6.0,
    "Hydro": 0.0,
    "Solar": 0.0,
    "Wind": 0.0,
    "Geothermal": 0.0,
}
CAPEX = {
    "Coal": 2800.0,
    "Oil": 2200.0,
    "Gas": 850.0,
    "Biomass": 4200.0,
    "Nuclear": 7500.0,
    "Hydro": 3200.0,
    "Solar": 900.0,
    "Wind": 1300.0,
    "Geothermal": 4500.0,
}


def mods_for(ptype: str, sid: str) -> dict:
    b = {
        "output_fraction": 0.0,
        "aux_load_fraction": 0.02,
        "efficiency_multiplier": 1.0,
        "ramp_limited": False,
        "allow_black_start": False,
        "tooling_label": "",
    }
    if sid == "Operational":
        b.update(
            {
                "output_fraction": 1.0,
                "aux_load_fraction": 0.04,
                "efficiency_multiplier": 1.0,
                "tooling_label": "full_dispatch",
            }
        )
    elif sid == "Standby":
        b.update(
            {
                "output_fraction": 0.0,
                "aux_load_fraction": 0.03,
                "efficiency_multiplier": 0.95,
                "tooling_label": "hot_standby",
            }
        )
    elif sid == "Maintenance":
        b.update(
            {
                "output_fraction": 0.0,
                "aux_load_fraction": 0.015,
                "efficiency_multiplier": 0.9,
                "tooling_label": "planned_outage",
            }
        )
    elif sid == "OutOfFuel":
        b.update(
            {
                "output_fraction": 0.0,
                "aux_load_fraction": 0.01,
                "efficiency_multiplier": 0.85,
                "tooling_label": "fuel_trip",
            }
        )
    elif sid == "StartingUp":
        b.update(
            {
                "output_fraction": 0.35,
                "aux_load_fraction": 0.06,
                "efficiency_multiplier": 0.88,
                "ramp_limited": True,
                "allow_black_start": True,
                "tooling_label": "sync_ramp",
            }
        )
    elif sid == "ShuttingDown":
        b.update(
            {
                "output_fraction": 0.15,
                "aux_load_fraction": 0.05,
                "efficiency_multiplier": 0.9,
                "ramp_limited": True,
                "tooling_label": "coast_down",
            }
        )
    elif sid == "Decommissioned":
        b.update(
            {
                "output_fraction": 0.0,
                "aux_load_fraction": 0.0,
                "efficiency_multiplier": 0.0,
                "tooling_label": "safstor_demolition",
            }
        )
    elif sid == "ExternalShutdown":
        b.update(
            {
                "output_fraction": 0.0,
                "aux_load_fraction": 0.02,
                "efficiency_multiplier": 0.92,
                "tooling_label": "grid_or_labor",
            }
        )
    elif sid == "ReducedCapacity":
        b.update(
            {
                "output_fraction": 0.65,
                "aux_load_fraction": 0.04,
                "efficiency_multiplier": 0.93,
                "tooling_label": "equipment_derate",
            }
        )
    elif sid == "OverCapacity":
        b.update(
            {
                "output_fraction": 1.08,
                "aux_load_fraction": 0.045,
                "efficiency_multiplier": 0.97,
                "tooling_label": "overload_excursion",
            }
        )
    elif sid == "EnvironmentalShutdown":
        if ptype in ("Solar", "Wind"):
            b.update(
                {
                    "output_fraction": 0.0,
                    "aux_load_fraction": 0.01,
                    "efficiency_multiplier": 0.9,
                    "tooling_label": "renewable_weather_window",
                }
            )
        elif ptype == "Hydro":
            b.update(
                {
                    "output_fraction": 0.0,
                    "aux_load_fraction": 0.02,
                    "efficiency_multiplier": 0.94,
                    "tooling_label": "fish_flood_drought",
                }
            )
        elif ptype == "Nuclear":
            b.update(
                {
                    "output_fraction": 0.0,
                    "aux_load_fraction": 0.08,
                    "efficiency_multiplier": 0.96,
                    "tooling_label": "intake_thermal_limit",
                }
            )
        else:
            b.update(
                {
                    "output_fraction": 0.0,
                    "aux_load_fraction": 0.03,
                    "efficiency_multiplier": 0.92,
                    "tooling_label": "permit_ambient",
                }
            )
    if ptype in ("Solar", "Wind") and sid == "OutOfFuel":
        b.update(
            {
                "output_fraction": 0.0,
                "aux_load_fraction": 0.01,
                "efficiency_multiplier": 1.0,
                "tooling_label": "na_map_to_reducedcapacity",
            }
        )
    return b


def instance(ptype: str) -> dict:
    z = {
        "reactor_units": 0,
        "unit_thermal_mw": 1.0,
        "turbine_units": 1,
        "hydro_head_m": 0.0,
        "hydro_design_flow_m3s": 0.0,
        "solar_array_area_m2": 0.0,
        "panel_nominal_efficiency": 0.0,
        "wind_rotor_diameter_m": 0.0,
        "wind_hub_height_m": 0.0,
        "geothermal_brine_inlet_c": 0.0,
        "geothermal_brine_flow_kg_s": 0.0,
        "fuel_feed_rate_kg_s": 0.0,
    }
    if ptype == "Nuclear":
        z["reactor_units"] = 1
        z["unit_thermal_mw"] = 3200.0
        z["turbine_units"] = 1
    if ptype == "Hydro":
        z["hydro_head_m"] = 85.0
        z["hydro_design_flow_m3s"] = 260.0
    if ptype == "Solar":
        z["solar_array_area_m2"] = 1_200_000.0
        z["panel_nominal_efficiency"] = 0.205
    if ptype == "Wind":
        z["wind_rotor_diameter_m"] = 136.0
        z["wind_hub_height_m"] = 80.0
    if ptype == "Geothermal":
        z["geothermal_brine_inlet_c"] = 165.0
        z["geothermal_brine_flow_kg_s"] = 420.0
    if ptype in ("Coal", "Oil", "Gas", "Biomass"):
        z["fuel_feed_rate_kg_s"] = {"Coal": 52.0, "Oil": 9.0, "Gas": 18.0, "Biomass": 7.0}[ptype]
    return z


def emissions(ptype: str) -> dict:
    e = {
        "co2_kg_per_mwh": 0.0,
        "nox_g_per_mwh": 0.0,
        "so2_g_per_mwh": 0.0,
        "particulate_g_per_mwh": 0.0,
        "thermal_outfall_kw_per_mw": 0.0,
    }
    if ptype == "Coal":
        e.update({"co2_kg_per_mwh": 820.0, "nox_g_per_mwh": 120.0, "so2_g_per_mwh": 80.0, "particulate_g_per_mwh": 15.0, "thermal_outfall_kw_per_mw": 450.0})
    elif ptype == "Gas":
        e.update({"co2_kg_per_mwh": 350.0, "nox_g_per_mwh": 35.0, "so2_g_per_mwh": 2.0, "thermal_outfall_kw_per_mw": 280.0})
    elif ptype == "Oil":
        e.update({"co2_kg_per_mwh": 650.0, "nox_g_per_mwh": 90.0, "so2_g_per_mwh": 300.0, "thermal_outfall_kw_per_mw": 380.0})
    elif ptype == "Biomass":
        e.update({"co2_kg_per_mwh": 30.0, "nox_g_per_mwh": 95.0, "so2_g_per_mwh": 40.0, "particulate_g_per_mwh": 25.0, "thermal_outfall_kw_per_mw": 320.0})
    elif ptype == "Nuclear":
        e.update({"thermal_outfall_kw_per_mw": 950.0})
    elif ptype == "Hydro":
        e.update({"co2_kg_per_mwh": 12.0})
    elif ptype in ("Solar", "Wind", "Geothermal"):
        e.update({"co2_kg_per_mwh": 8.0 if ptype != "Geothermal" else 45.0})
    return e


def ramp_up(ptype: str) -> float:
    return {"Nuclear": 25.0, "Hydro": 80.0, "Coal": 40.0, "Gas": 120.0, "Solar": 300.0, "Wind": 60.0}.get(ptype, 35.0)


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    out = root / "assets" / "config" / "power" / "plant_definitions.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    plants = []
    for ptype, pid, blurb in PLANTS:
        caps = {
            "is_steam_cycle": ptype in ("Coal", "Oil", "Gas", "Biomass", "Nuclear", "Geothermal"),
            "is_nuclear_containment": ptype == "Nuclear",
            "is_variable_renewable": ptype in ("Solar", "Wind"),
        }
        sm = {s: mods_for(ptype, s) for s in STATUSES}
        min_stable = 0.18 if ptype in ("Coal", "Nuclear", "Hydro", "Geothermal") else 0.05
        plants.append(
            {
                "id": pid,
                "display_name": pid.replace("_v1", "").replace("_", " ").title(),
                "plant_type": ptype,
                "narrative": {
                    "short_description": blurb,
                    "operator_notes": f"Registry id `{pid}`; instance overrides in saves.",
                    "risk_summary": "Capability-scoped failures (`SteamCycle`, `ContainmentBuilding`, `VariableRenewable`).",
                },
                "output_model": {
                    "nameplate_mw": NAMEPLATE[ptype],
                    "base_efficiency_factor": EFF[ptype],
                    "part_load_curve_id": f"{ptype.lower()}_generic_partload_v1",
                    "min_stable_output_fraction": min_stable,
                    "ramp_up_mw_per_minute": ramp_up(ptype),
                    "ramp_down_mw_per_minute": 40.0,
                    "inertia_constant_h": 4.5 if ptype in ("Coal", "Nuclear", "Hydro") else 2.0,
                },
                "operational": {
                    "status_modifiers": sm,
                    "transition_hints": [
                        {"from": "Standby", "to": "StartingUp", "trigger": "dispatch", "typical_duration_s": 300.0},
                        {
                            "from": "StartingUp",
                            "to": "Operational",
                            "trigger": "synchronize",
                            "typical_duration_s": 2400.0 if ptype == "Nuclear" else 600.0,
                        },
                    ],
                },
                "instance_template": instance(ptype),
                "capabilities": caps,
                "emissions": emissions(ptype),
                "economics": {
                    "capex_per_kw": CAPEX[ptype],
                    "fixed_om_per_kw_year": 28.0,
                    "variable_om_per_mwh": 3.2,
                    "fuel_cost_per_mwh_thermal": FUEL[ptype],
                    "expected_availability": 0.87 if ptype in ("Solar", "Wind") else 0.91,
                },
                "research": {
                    "required_research_ids": ([] if ptype not in ("Nuclear",) else ["nuclear_gen_ii_plus_v1"]),
                    "unlock_notes": "Research gates are soft until the tree exists.",
                },
                "grid_interface": {
                    "power_factor": 0.95,
                    "inrush_multiplier": {"Solar": 1.15, "Wind": 1.25}.get(ptype, 1.05),
                    "allow_grid_forming": ptype in ("Hydro", "Nuclear", "Coal", "Gas"),
                },
            }
        )
    doc = {"schema_version": 1, "plants": plants}
    out.write_text(json.dumps(doc, indent=2), encoding="utf-8")
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
