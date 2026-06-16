> **Superseded as authority:** split into [`research_capability_ecosystem_runbook_v1.md`](research_capability_ecosystem_runbook_v1.md), [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md), [`infrastructure_resilience_and_failure_runbook_v1.md`](infrastructure_resilience_and_failure_runbook_v1.md) under [`infrastructure_and_research_orchestrator_v1.md`](infrastructure_and_research_orchestrator_v1.md). Keep this file as an archival bundle only.

research_capability_ecosystem_runbook_v1.md

Version: v1.0.0
Parent:

simulation_expansion_orchestrator_v1.md

Related:

logistics_simulation_runbook_v1.md
infrastructure_networks_runbook_v1.md
education_population_runbook_v1.md
industrial_simulation_runbook_v1.md
1. Purpose

Replace traditional linear “tech tree” progression with:

capability emergence
+
industrial dependency
+
organizational development
+
knowledge ecosystems

Research is treated as:

infrastructure
institutions
industrial maturity
experimentation
logistics capability
doctrine pressure
production experience

NOT:

isolated unlock buttons.
2. Core Philosophy

Avoid:

Mining II
→ Steel III
→ Tank IV

Prefer:

material capability
+
manufacturing sophistication
+
organizational doctrine
+
resource availability
+
practical experience
=
new technological possibilities
3. System Layers
Layer	Purpose
Knowledge Domains	broad theoretical understanding
Industrial Capability	ability to mass produce
Practical Experience	operational usage feedback
Institutions	research and education generation
Doctrine	strategic prioritization
Discovery Graph	emergent unlock conditions
Production Maturity	quality/reliability scaling
4. Knowledge Domains
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KnowledgeDomain {
    Metallurgy,
    Chemistry,
    Agriculture,
    Hydrology,
    Logistics,
    CivilEngineering,
    MaterialsScience,
    Electronics,
    Thermodynamics,
    Combustion,
    Aerodynamics,
    Communications,
    Medicine,
    EnergySystems,
    Computing,
}
5. Capability Progression

Research is multidimensional.

pub struct KnowledgeProgress {
    pub theory: f32,
    pub practical: f32,
    pub industrialization: f32,
}
Example

A faction may have:

high metallurgy theory
low industrialization

meaning:

advanced prototypes possible
mass production unreliable
6. Institutions

Research generation comes from:

Institution	Contribution
universities	theory
factories	practical knowledge
laboratories	experimentation
military	doctrine pressure
logistics networks	operational optimization
hospitals	medical advancement
power grids	electrification support
7. Discovery Graph

Technologies are emergent possibilities.

Example
precision tooling
+
advanced metallurgy
+
stable electricity
+
industrial labor surplus
=
advanced turbine manufacturing possible

NOT:
“unlock turbine tech.”

8. Research Pressure

Knowledge development is influenced by:

Pressure	Result
warfare	rapid weapons innovation
famine	agriculture focus
logistics collapse	transport optimization
fuel shortages	energy alternatives
disasters	engineering adaptation
9. Reverse Engineering

Captured systems can accelerate:

pub struct ReverseEngineeringSource {
    pub source_faction: Entity,
    pub technology_complexity: f32,
    pub compatibility: f32,
}
10. Knowledge Diffusion

Knowledge spreads through:

trade
migration
espionage
education
alliances
captured infrastructure
11. Production Maturity

Unlocking capability ≠ reliable production.

pub struct ProductionMaturity {
    pub defect_rate: f32,
    pub throughput_efficiency: f32,
    pub maintenance_burden: f32,
}
12. Organizational Friction

Large organizations suffer:

bureaucracy
inertia
political resistance
retraining costs
13. Doctrine Systems

Doctrine modifies:

funding
priorities
deployment patterns
specialization

Example:

pub enum StrategicDoctrine {
    DefensiveEngineering,
    MechanizedWarfare,
    NavalProjection,
    IndustrialExpansion,
    EcologicalStability,
}
14. Gameplay UX

Research gameplay should focus on:

planning ecosystems
balancing priorities
responding to crises
managing institutions
industrial specialization

NOT:

clicking static nodes.
15. Strategic Overlays

Research visualization uses overlays:

industrial concentration
educational density
logistics sophistication
electrification coverage
production maturity
16. Long-Term Goals

Supports:

asymmetric development
industrial divergence
emergent technological eras
infrastructure dependency
doctrine-driven economies
infrastructure_construction_runbook_v1.md

Version: v1.0.0
Parent:

simulation_expansion_orchestrator_v1.md

Related:

logistics_simulation_runbook_v1.md
terrain_unification_runbook_v1.md
weather_simulation_runbook_v1.md
fire_ecology_simulation_runbook_v1.md
1. Purpose

Define the infrastructure, construction, planning, and spatial engineering systems.

The game is treated as:

territorial systems engineering

rather than:

isolated building placement
abstract economy management
2. Core Philosophy

Infrastructure is:

spatial
physical
logistical
terrain-dependent
damageable
maintainable
interconnected
3. Major Infrastructure Categories
Category	Examples
Transport	roads, rails, bridges
Utilities	power lines, substations, pipelines
Industry	factories, refineries, mills
Civil	housing, hospitals, schools
Military	forts, trenches, depots
Ecology	firebreaks, forestry zones
Hydrology	canals, levees, drainage
4. Planning First Workflow

All construction follows:

plan
→ survey
→ allocate logistics
→ construct
→ maintain
→ degrade
→ repair

NOT instant spawning.

5. Infrastructure Graph Layer

Core engine system.

pub enum InfrastructureNetworkType {
    Roads,
    Rail,
    Power,
    Pipelines,
    Communications,
}
6. Graph Structure
pub struct InfrastructureNode {
    pub id: u64,
    pub position: Vec2,
}

pub struct InfrastructureEdge {
    pub from: u64,
    pub to: u64,

    pub throughput: f32,
    pub integrity: f32,
    pub maintenance_cost: f32,
}
7. Road System
pub enum RoadClass {
    DirtTrack,
    GravelRoad,
    PavedRoad,
    Highway,
    MilitaryRoad,
}

Properties:

throughput
weather resistance
mud susceptibility
maintenance
width
speed modifier
8. Rail System

Rail acts as:

high throughput
high efficiency
rigid routing
strategic vulnerability

Supports:

depots
yards
switching
bridges
tunnels
9. Power Grid System

Power should exist physically.

Grid Components
pub enum PowerLineClass {
    Local,
    Regional,
    HighVoltage,
}
Power Infrastructure
Structure	Purpose
power plant	generation
transformer	voltage management
substation	distribution
transmission line	long-distance transfer
battery bank	storage
switchyard	routing
10. Utility Placement UX

Power lines/pipes use:

spline placement
snapping
terrain-aware routing
obstacle detection
corridor planning
11. Construction States
pub enum ConstructionState {
    Planned,
    Surveying,
    Clearing,
    UnderConstruction,
    Operational,
    Damaged,
    Abandoned,
}
12. Logistics Coupling

Construction consumes:

labor
fuel
concrete
steel
machinery
transport capacity

No free placement.

13. Terrain Coupling

Terrain affects:

road speed
rail grades
landslide risk
flood exposure
erosion
bridge requirements
14. Weather Coupling

Weather impacts:

mud
flooding
visibility
construction speed
road degradation
power outages
fire risk
15. Fortification System

Fortifications are terrain engineering.

Fortification Types
pub enum FortificationType {
    Trench,
    Berm,
    Bunker,
    DragonTeeth,
    Minefield,
    RazorWire,
    ArtilleryPit,
    HardenedDepot,
}
16. Terrain Modification

Fortifications may alter:

drainage
erosion
mobility
vegetation
visibility
17. Urban Development

Cities emerge from:

transport access
utilities
industry
labor availability
terrain suitability
defense
economy

NOT only direct building placement.

18. Resource Nodes

Resource extraction depends on:

geology
transport
power
labor
environmental conditions
19. Suggested Flow Systems

The engine may suggest:

efficient logistics routes
rail corridors
utility corridors
firebreak placement
congestion mitigation

without removing player agency.

20. Build Modes
Mode	Purpose
Transport	roads/rail
Utilities	power/pipes
Industry	factories
Military	defenses
Ecology	forestry/firebreaks
Civil	housing/services
21. Strategic Overlay Visualization

Critical overlays:

Overlay	Purpose
logistics throughput	bottlenecks
power flow	outages
congestion	transport load
fire risk	hazard
flood risk	terrain danger
maintenance	infrastructure stress
construction supply	stalled projects

Prefer:

world overlays
GPU compositing
contour visualization

NOT excessive floating windows.

22. Construction UX Rules
Planning

Player drafts:

corridors
districts
networks
fortification lines

before execution.

Ghost Placement

Support:

phased construction
priorities
budgeting
deferred execution
Contextual Tools

Tools should expose:

costs
throughput
terrain penalties
predicted congestion
logistics requirements
23. Icons & Visual Language

Icons should prioritize:

readability
silhouette recognition
category consistency
strategic clarity
Recommended Categories
Category	Visual Language
roads	linear
rail	segmented
power	lightning/grid
logistics	arrows/crates
military	angular/fortified
ecology	organic
fire	heat/flame
utilities	pipe/cable
24. UI Direction

Gameplay UI should emphasize:

overlays
contextual inspectors
strategic readability
minimal clutter

Avoid:

excessive floating windows
duplicated controls
permanent debug spam



26. Critical Design Rule

Infrastructure is NOT decorative.

It is:

simulated
vulnerable
logistical
strategic
environmentally coupled

and should meaningfully affect:

warfare
economy
ecology
population
industrial capability
political stability.


infrastructure_resilience_and_failure_runbook_v1.md

Version: v1.0.0
Parent:

infrastructure_construction_runbook_v1.md

Related:

logistics_simulation_runbook_v1.md
weather_simulation_runbook_v1.md
fire_ecology_simulation_runbook_v1.md
hydrology_simulation_runbook_v1.md
population_simulation_runbook_v1.md
1. Purpose

Define:

infrastructure degradation
repair logistics
network failures
sabotage
resilience systems
cascading utility collapse
disaster response
ecological recovery

Infrastructure is treated as:

persistent physical systems under stress

not static indestructible entities.

2. Core Philosophy

Every infrastructure system should support:

construction
→ operation
→ degradation
→ overload
→ damage
→ repair
→ adaptation
→ replacement
3. Major Failure Categories
Failure Type	Examples
Mechanical	wear, fatigue
Environmental	erosion, flood, fire
Operational	overload, congestion
Human	sabotage, neglect
Military	bombing, shelling
Ecological	root destabilization, landslides
4. Infrastructure Health Model
#[derive(Clone, Copy, Debug)]
pub struct InfrastructureCondition {
    pub structural_integrity: f32,
    pub operational_efficiency: f32,
    pub maintenance_debt: f32,
    pub overload_stress: f32,
    pub environmental_damage: f32,
}
5. Aging Model

Infrastructure slowly accumulates:

fatigue
corrosion
thermal stress
erosion
vegetation intrusion
foundation instability
Aging Drivers
Driver	Examples
weather	freeze/thaw
heavy usage	road wear
overload	transformers
moisture	tunnel seepage
salt exposure	coastal corrosion
neglect	maintenance debt
6. Maintenance Crew System

Maintenance is physical logistics work.

Crew Types
pub enum MaintenanceCrewType {
    RoadRepair,
    RailRepair,
    Electrical,
    PipelineRepair,
    StructuralEngineering,
    FireRecovery,
    FloodRecovery,
}
Crew Requirements

Crews require:

transport access
fuel
spare parts
labor
safety conditions
Crew Workflow
damage detected
→ dispatch crew
→ route planning
→ resource delivery
→ repair work
→ network restoration
7. Maintenance Prioritization

Priorities may include:

Priority	Example
military	frontline rail
civilian	hospitals
industrial	power grid
economic	ports
ecological	levees/firebreaks
8. Deferred Maintenance

Ignoring maintenance creates:

increasing repair cost
reduced throughput
catastrophic failure probability
9. Sabotage System

Sabotage targets:

bridges
substations
rail junctions
pipelines
depots
telecom nodes
Sabotage Characteristics
pub struct SabotageEvent {
    pub severity: f32,
    pub stealth: f32,
    pub infrastructure_target: Entity,
    pub faction: Option<Entity>,
}
10. Counter-Sabotage

Detection influenced by:

patrols
surveillance
population loyalty
intelligence networks
redundancy
11. Utility Overload System

Utilities can exceed safe operating limits.

Example Causes
Cause	Result
power demand spike	transformer overheating
rail congestion	routing collapse
excessive trucks	road damage
flood pumping	grid overload
12. Load Model
pub struct UtilityLoadState {
    pub current_load: f32,
    pub safe_capacity: f32,
    pub overload_factor: f32,
}
13. Cascading Failure System

Failures propagate through networks.

Example
substation overload
→ neighboring reroute
→ neighboring overload
→ rolling blackout
→ industrial shutdown
→ water pumping failure
→ urban disruption
14. Cascading Blackouts

Electrical networks support:

dynamic load balancing
rolling outages
brownouts
emergency isolation
Blackout Consequences
Sector	Impact
industry	production halt
logistics	rail slowdown
population	unrest
hospitals	mortality risk
military	radar/comms loss
15. Dynamic Rerouting

Networks automatically seek alternative paths.

Routing Layers
Network	Routing
roads	pathfinding
rail	graph dispatch
power	flow balancing
pipelines	pressure routing
Dynamic Routing Goals
maintain service
reduce overload
avoid damage zones
preserve strategic supply
16. Infrastructure Redundancy

Redundant systems improve resilience.

Examples
Redundancy	Benefit
dual rail lines	rerouting
ring grids	blackout isolation
bypass roads	congestion reduction
reserve substations	emergency load
17. Bridge Collapse System

Bridges are strategic choke points.

Failure Causes
Cause	Example
overload	excessive armor traffic
sabotage	explosives
flood erosion	pier undermining
fire	steel weakening
bombing	structural failure
Bridge State
pub enum BridgeCondition {
    Operational,
    Damaged,
    StructurallyUnsafe,
    Collapsed,
}
18. Bridge Collapse Consequences

Collapse may:

sever logistics
isolate regions
reroute armies
increase congestion
disrupt utilities
19. Tunnel Flooding System

Tunnels interact with:

groundwater
rainfall
nearby rivers
pumping systems
power availability
Flood Sequence
heavy rainfall
→ seepage increase
→ pump overload
→ blackout
→ tunnel flooding
→ route shutdown
Tunnel State
pub struct TunnelHydrologyState {
    pub water_level: f32,
    pub seepage_rate: f32,
    pub pump_capacity: f32,
}
20. Ecological Restoration System

Infrastructure damage can trigger ecological recovery.

Examples
Damage	Ecological Result
abandoned rail	vegetation corridor
destroyed city	regrowth
flooded mine	wetland formation
firebreak neglect	forest return
Restoration Drivers
Driver	Impact
rainfall	revegetation
soil quality	biomass growth
seed dispersal	succession
pollution	suppression
grazing	recovery limits
21. Restoration States
pub enum EcologicalRecoveryStage {
    Sterile,
    PioneerGrowth,
    Grassland,
    YoungForest,
    MatureForest,
}
22. Infrastructure ↔ Ecology Feedback

Infrastructure influences ecology:

fragmentation
runoff
pollution
fire spread
species migration

Ecology influences infrastructure:

root damage
erosion control
flood mitigation
fire risk
visibility
23. Fire Recovery Integration

Burned areas may:

destabilize slopes
alter hydrology
increase sediment flow
reduce road stability
24. Long-Term Simulation Goals

Supports:

wartime infrastructure collapse
industrial decay
environmental disasters
climate stress
strategic bombing campaigns
resilient city planning
25. GPU Simulation Opportunities

Candidate GPU field simulations:

System	GPU Suitability
blackout propagation	high
flood spread	high
wildfire spread	high
traffic density	high
erosion	high
vegetation recovery	medium
26. ECS vs GPU Rule

Keep authoritative:

logistics state
ownership
strategic routing
repair jobs
economy

on CPU/ECS.

Use GPU for:

propagation fields
visualization
hazard diffusion
large-scale environmental simulation
27. Overlay Visualization

Recommended overlays:

Overlay	Purpose
maintenance debt	infrastructure health
blackout zones	power outages
rerouting congestion	traffic stress
flood risk	tunnel danger
sabotage risk	security
ecological recovery	regrowth
28. UX Principles

Players should:

anticipate failures
build redundancy
prioritize repairs
understand cascading consequences

Avoid:

invisible failures
arbitrary destruction
opaque routing behavior
29. AI Strategic Behavior

AI factions should:

target bottlenecks
defend substations
repair key corridors
build redundant logistics
exploit ecological damage
30. Critical Design Rule

Infrastructure failure should create:

emergent strategic geography

where:

destroyed bridges reshape fronts
blackouts cripple industry
floods isolate regions
ecological recovery alters terrain
logistics reroute dynamically

instead of scripted map events.