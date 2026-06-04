ECONOMIC + LOGISTICS SIMULATION KERNEL (CORE ENGINE)
0. Core Principle

Everything becomes:

Production → Transport → Consumption → Feedback

No exceptions.

Buildings, cities, armies, railways, ports — all are just nodes in this loop.

1. SYSTEM ARCHITECTURE
            +-------------------+
            |  Population       |
            +---------+---------+
                      |
                      v
            +-------------------+
            |  Demand System    |
            +---------+---------+
                      |
                      v
            +-------------------+
            |  Production       |
            +---------+---------+
                      |
                      v
            +-------------------+
            |  Logistics Graph  |
            +---------+---------+
                      |
                      v
            +-------------------+
            |  Consumption      |
            +---------+---------+
                      |
                      v
            +-------------------+
            |  Feedback System  |
            +-------------------+
2. CORE DATA MODEL
2.1 Resources (Everything in the world)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Resource {
    Food,
    Water,
    Wood,
    Steel,
    Concrete,
    Fuel,
    Electricity,
    Electronics,
    Machinery,
    Vehicles,
    Ammunition,
}
2.2 Stockpiles

Every entity can store resources.

#[derive(Component)]
pub struct Stockpile {
    pub capacity: f32,
    pub stored: HashMap<Resource, f32>,
}
2.3 Production Nodes
#[derive(Component)]
pub struct Producer {
    pub outputs: Vec<(Resource, f32)>,
    pub inputs: Vec<(Resource, f32)>,
    pub rate: f32,
}

Example:

Steel Mill:
  input: Iron, Coal
  output: Steel
2.4 Consumer Nodes
#[derive(Component)]
pub struct Consumer {
    pub demand: Vec<(Resource, f32)>,
    pub priority: f32,
}
3. DEMAND SYSTEM (THE BRAIN)

Everything starts here.

3.1 Global Demand State
#[derive(Resource)]
pub struct GlobalDemand {
    pub demand: HashMap<Resource, f32>,
}
3.2 Demand Calculation
pub fn update_demand(
    consumers: Query<&Consumer>,
    mut global: ResMut<GlobalDemand>,
) {
    global.demand.clear();

    for c in &consumers {
        for (res, amount) in &c.demand {
            *global.demand.entry(*res).or_default() += amount * c.priority;
        }
    }
}
3.3 Key Insight

Demand is NOT per building.

It is:

population driven
industry driven
war driven
logistics failure driven
4. PRODUCTION SYSTEM
4.1 Production Tick
pub fn production_system(
    mut producers: Query<(&mut Stockpile, &Producer)>,
) {
    for (mut stock, prod) in &mut producers {
        // check inputs
        if has_inputs(&stock, &prod.inputs) {
            consume_inputs(&mut stock, &prod.inputs);

            for (res, amount) in &prod.outputs {
                *stock.stored.entry(*res).or_default() += amount * prod.rate;
            }
        }
    }
}
4.2 Production Dependency Graph

This is critical:

Iron Mine → Steel Mill → Factory → City Consumption

Everything becomes a DAG.

5. LOGISTICS GRAPH SYSTEM (CORE INNOVATION)

This is what makes your simulation “alive”.

5.1 Transport Edge
#[derive(Component)]
pub struct LogisticsEdge {
    pub from: Entity,
    pub to: Entity,
    pub capacity: f32,
    pub cost: f32,
}
5.2 Transport Node
#[derive(Component)]
pub struct LogisticsNode {
    pub stockpile: Stockpile,
}
5.3 Shipment System
pub struct Shipment {
    pub resource: Resource,
    pub amount: f32,
    pub from: Entity,
    pub to: Entity,
    pub progress: f32,
}
5.4 Routing Logic (simple version)
pub fn route_shipment(shipment: &mut Shipment) {
    shipment.progress += 0.01;
}

(Replace later with A* over logistics graph)

5.5 Real Version (Important)

You eventually replace routing with:

Dijkstra(cost = congestion + distance + terrain + fuel usage)
6. ECONOMIC FEEDBACK LOOP

This is where “city intelligence” comes from.

6.1 Price Signal Model
#[derive(Resource)]
pub struct Market {
    pub price: HashMap<Resource, f32>,
}
6.2 Price Update Rule
pub fn update_prices(
    demand: Res<GlobalDemand>,
    mut market: ResMut<Market>,
) {
    for (res, d) in &demand.demand {
        let price = market.price.entry(*res).or_insert(1.0);

        *price = (*price * 0.95) + (*d * 0.05);
    }
}
6.3 Why This Matters

Now:

shortage → price increase
price increase → production increase
production increase → logistics pressure
logistics failure → local collapse

You now have emergent economy.

7. DISTRICT → ECONOMY BRIDGE

This connects your previous system.

7.1 District Economic Output
pub struct DistrictEconomy {
    pub workforce: f32,
    pub productivity: f32,
}
7.2 Conversion
population → workforce
workforce → production capacity
accessibility → efficiency multiplier
8. MCP INTEGRATION (IMPORTANT)

This system generates asset requests.

8.1 Example Trigger
if steel_shortage > threshold {
    send_mcp_request(
        "blender.generate",
        BuildingSpec::SteelPlant
    );
}
8.2 Output Flow
Economy detects demand
        ↓
MCP request generated
        ↓
Factory / building generated
        ↓
Inserted into simulation
9. MILITARY EXTENSION (same system)

Military is just:

Consumer (ammo, fuel, vehicles)
Producer (war factories)
Logistics (front supply lines)

No separate system required.

10. WHAT THIS SYSTEM ACTUALLY GIVES YOU

You now have a unified simulation where:

Cities:
grow due to demand
Industry:
appears due to shortages
Transport:
becomes critical bottleneck
War:
collapses logistics, not just HP bars
Buildings:
are outputs of simulation, not placed objects
11. KEY ARCHITECTURAL SHIFT

You are no longer building:

a city builder

You are building:

a world-state economic simulator with visual representationFRONTLINE + CONFLICT SIMULATION SYSTEM
0. Core Idea

Forget unit RTS.

You simulate:

pressure → logistics strain → front instability → territorial shift

War is NOT units fighting.

War is:

supply failure
production imbalance
transport disruption
attrition zones

Units are just a visualization layer.

1. WORLD STRUCTURE
1.1 Territorial Graph (NOT TILEMAP)
#[derive(Component)]
pub struct TerritoryNode {
    pub id: TerritoryId,
    pub owner: FactionId,
    pub stability: f32,
    pub industry_value: f32,
    pub population: f32,
}
1.2 Connections
#[derive(Component)]
pub struct TerritoryEdge {
    pub from: TerritoryId,
    pub to: TerritoryId,
    pub transport_capacity: f32,
    pub contested: bool,
}
1.3 Factions
#[derive(Component)]
pub struct Faction {
    pub id: FactionId,
    pub military_strength: f32,
    pub logistics_efficiency: f32,
    pub industrial_base: f32,
}
2. FRONT SYSTEM (CORE CONCEPT)

A front is NOT a line.

It is a pressure boundary between territories.

2.1 Front Definition
#[derive(Component)]
pub struct Front {
    pub a: TerritoryId,
    pub b: TerritoryId,
    pub pressure_a: f32,
    pub pressure_b: f32,
    pub stability: f32,
}
2.2 Front Pressure Model
pressure =
    military_presence
  + logistics_flow
  + terrain advantage
  - supply efficiency
  - infrastructure strength
2.3 Update System
pub fn update_fronts(
    mut fronts: Query<&mut Front>,
    factions: Query<&Faction>,
    territories: Query<&TerritoryNode>,
) {
    for mut f in &mut fronts {
        let a = territories.get(f.a).unwrap();
        let b = territories.get(f.b).unwrap();

        let fa = factions.get(a.owner).unwrap();
        let fb = factions.get(b.owner).unwrap();

        f.pressure_a = fa.military_strength + a.industry_value;
        f.pressure_b = fb.military_strength + b.industry_value;

        f.stability = (f.pressure_a - f.pressure_b).abs();
    }
}
3. TERRITORIAL CONTROL SYSTEM
3.1 Ownership Drift

Territories don’t flip instantly.

They drift.

pub fn update_territory_control(
    mut territories: Query<&mut TerritoryNode>,
    fronts: Query<&Front>,
) {
    for mut t in &mut territories {
        let influence = calculate_influence(t.id, &fronts);

        t.stability -= influence.enemy_pressure;
        t.stability += influence.friendly_pressure;

        if t.stability < 0.2 {
            t.owner = influence.strongest_faction;
            t.stability = 0.5;
        }
    }
}
4. SUPPLY LINES (THIS IS THE REAL WAR SYSTEM)

War is decided here, not combat.

4.1 Supply Route
#[derive(Component)]
pub struct SupplyLine {
    pub from: TerritoryId,
    pub to: TerritoryId,
    pub throughput: f32,
    pub disruption: f32,
}
4.2 Supply Impact Model
combat_effectiveness =
    base_strength
  × supply_ratio
  × morale
  × logistics_quality
4.3 Supply Collapse Logic
if supply_ratio < 0.3 {
    unit_strength *= 0.5;
}
5. ECONOMY → WAR LINK (CRITICAL)

Your previous system plugs directly in.

5.1 Industrial Output feeds military strength
steel → weapons
fuel → mobility
food → morale
electronics → coordination
5.2 Military drains economy
army upkeep =
    fuel + food + ammo + maintenance
6. FRONT COLLAPSE MODEL

This is the emergent war logic.

6.1 Collapse Equation
front_stability =
    supply_flow
  + infrastructure_strength
  + faction_control
  - enemy_pressure
6.2 Collapse Trigger
if front.stability < threshold {
    trigger_territory_shift();
}
7. TERRITORIAL SHIFT EVENT

This is your “war moment”.

pub fn territory_shift(front: &mut Front) {
    front.b = front.a;
}

But in full system:

buildings change ownership
supply chains reroute
industries halt
MCP requests trigger rebuilds
8. MCP INTEGRATION (IMPORTANT)

War directly generates construction requests.

8.1 Example: destroyed city zone
{
  "tool": "blender.generate",
  "input": {
    "Building": {
      "type": "ruined_residential",
      "damage": 0.8,
      "style": "industrial_warzone"
    }
  }
}
8.2 Example: new fortification
{
  "tool": "blender.generate",
  "input": {
    "Building": {
      "type": "bunker",
      "fortification_level": 5
    }
  }
}
9. VISUALIZATION LAYER (IMPORTANT CLARITY)

You do NOT simulate units directly.

You optionally spawn:

battalions (visual proxy)
convoys (logistics visualization)
front lines (heatmaps)

Everything is derived.

10. CORE SYSTEM LOOP

This is your full game loop now:

ECONOMY
   ↓
INDUSTRY OUTPUT
   ↓
LOGISTICS GRAPH
   ↓
FRONT PRESSURE
   ↓
TERRITORIAL CONTROL
   ↓
ECONOMIC SHIFT
   ↓
MCP BUILD REQUESTS
   ↓
WORLD UPDATES
   ↺
11. WHAT YOU NOW HAVE (IMPORTANT)

This completes your architecture:

✔ Cities grow organically
✔ Industry emerges from demand
✔ Logistics determines survival
✔ War is emergent, not scripted
✔ Territory shifts dynamically
✔ MCP generates world changes
✔ Visuals are just projection of simulation
12. FINAL SYSTEM REALIZATION

You are no longer building:

a strategy game

You are building:

a persistent simulated world with economic + territorial physics