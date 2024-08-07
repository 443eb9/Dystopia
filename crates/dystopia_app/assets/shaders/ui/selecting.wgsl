#import bevy_ui::ui_vertex_output::UiVertexOutput

struct BodySelectingIconMaterial {
    line_color: vec3f,
    line_width: f32,
}

@group(1) @binding(0) var<uniform> material: BodySelectingIconMaterial;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4f {
    let abs_uv = abs(in.uv * 2. - 1.);
    let fmtted_uv = in.size * abs_uv;

    if fmtted_uv.x > (in.size.x - material.line_width) && abs_uv.y > 0.5
       || fmtted_uv.y > (in.size.y - material.line_width) && abs_uv.x > 0.5 {
        return vec4f(material.line_color, 1.);
    } else {
        return vec4f(0.);
    }
}
