#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct RockyBodyMaterial {
    color: vec3f,
}

@group(2) @binding(0) var<uniform> material: RockyBodyMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let d = in.uv - vec2f(0.5);
    if dot(d, d) < 0.25 {
        return vec4f(material.color, 1.);
    } else {
        return vec4f(0.);
    }
}
