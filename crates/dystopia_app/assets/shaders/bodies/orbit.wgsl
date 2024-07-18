#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct OrbitMaterial {
    color: vec3f,
    width: f32,
    radius: f32,
}

@group(2) @binding(0) var<uniform> material: OrbitMaterial;

// Synced with dystopia_core::cosmos::ORBIT_MESH_SCALE
const ORBIT_MESH_SCALE: f32 = 1.1;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let d2 = length((in.uv - vec2f(0.5)) * ORBIT_MESH_SCALE) * 2.;
    let t = material.width / material.radius * 0.5;

    if d2 > 1. - t && d2 < 1. + t {
        return vec4f(material.color.rgb, 0.7);
    } else {
        return vec4f(0.);
    }
}
