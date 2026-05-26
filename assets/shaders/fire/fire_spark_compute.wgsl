// FX-FIRE-SPARK-002 (Phase B) — legacy compute_expanse_BASE_A.glsl port.
// pos.w = lifetime; attractors update velocity after integration + respawn.

struct SparkComputeUniforms {
    delta_time: f32,
    instance_count: u32,
    attractor_count: u32,
    lifetime_decay: f32,
    respawn_life: f32,
    _pad: f32,
}

struct GpuParticleInstance {
    world_xyz_heat: vec4<f32>,
    ember_class_radius_smoke: vec4<f32>,
}

struct SparkSimState {
    pos: vec4<f32>,
    vel: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: SparkComputeUniforms;
@group(1) @binding(0) var<storage, read> instances: array<GpuParticleInstance>;
@group(2) @binding(0) var<storage, read_write> spark_state: array<SparkSimState>;
@group(3) @binding(0) var<storage, read> attractors: array<vec4<f32>>;

const MAX_ATTRACTORS: u32 = 24u;

fn nearest_attractor(origin: vec3<f32>, count: u32) -> vec3<f32> {
    if count == 0u {
        return origin;
    }
    var best = attractors[0].xyz;
    var best_d = distance(origin, best);
    for (var j: u32 = 1u; j < count; j = j + 1u) {
        let p = attractors[j].xyz;
        let d = distance(origin, p);
        if d < best_d {
            best_d = d;
            best = p;
        }
    }
    return best;
}

@compute @workgroup_size(64, 1, 1)
fn advect_sparks(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i >= params.instance_count {
        return;
    }

    let row = instances[i];
    let origin = row.world_xyz_heat.xyz;
    var state = spark_state[i];
    let dt = max(params.delta_time, 0.0001);
    let dt2 = dt * dt;

    // Legacy order: integrate with previous velocity, decay life, respawn, then attractors → vel.
    let life = state.pos.w - params.lifetime_decay * dt;
    state.pos = vec4(state.pos.xyz - state.vel.xyz * dt, life);

    if state.pos.w <= 0.0 {
        let spawn = nearest_attractor(origin, params.attractor_count);
        state.pos = vec4(spawn, params.respawn_life);
        state.vel = vec4(state.vel.xyz * 0.9, 0.0);
    }

    var val = state.vel;
    let count = min(params.attractor_count, MAX_ATTRACTORS);
    for (var j: u32 = 0u; j < count; j = j + 1u) {
        let att = attractors[j];
        let dist = att.xyz - state.pos.xyz;
        let d2 = max(dot(dist, dist), 0.01);
        val = vec4(
            val.xyz + ((dt2 * att.w * 26.0) * normalize(dist)) / (d2 * 260.0),
            0.0,
        );
    }

    state.vel = val;
    spark_state[i] = state;
}
