#import bevy_render::{view::View, globals::Globals}

struct Tilemap {
    tile_render_size: vec2f,
    world_from_model: mat4x4f,
    tint: vec4f,
}

struct TilemapVertexInput {
    @builtin(vertex_index) v_index: u32,
    @location(0) position: vec3f,
    @location(1) color: vec4f,
    /// If not animated: `[texture_index, atlas_index, special, 0]`
    ///
    /// If animated: `[start, len, special, offset_milisec]`
    @location(2) atlas_index: vec4u,
    @location(3) tile_index: vec2i,
}

struct TilemapVertexOutput {
    @builtin(position) position_cs: vec4f,
    @location(0) tint: vec4f,
    @location(1) uv: vec2f,
    @location(2) texture_index: u32,
}

struct TilemapTextureDescriptor {
    tile_count: vec2u,
}

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<uniform> globals: Globals;
@group(0) @binding(2) var<uniform> tilemap: Tilemap;
@group(0) @binding(3) var texture: texture_2d_array<f32>;
@group(0) @binding(4) var texture_sampler: sampler;
@group(0) @binding(5) var<storage> texture_desc: array<TilemapTextureDescriptor>;
@group(0) @binding(6) var<storage> animations: array<u32>;

fn decode_atlas_and_flip_uv(atlas_index: u32, uv: ptr<function, vec2f>) -> u32 {
    let flip = atlas_index >> 30;
    if (flip & 1u) != 0u {
        (*uv).y = 1. - (*uv).y;
    }
    if (flip & 2u) != 0u {
        (*uv).x = 1. - (*uv).x;
    }
    return atlas_index & 0x3FFFFFFF;
}

fn vert_offset(corner: u32) -> vec2f {
    var tranl = array<vec2<f32>, 4>(
        vec2<f32>(0., 0.),
        vec2<f32>(0., 1.),
        vec2<f32>(1., 1.),
        vec2<f32>(1., 0.),
    );
    return tranl[corner];
}

fn vert_uv(corner: u32) -> vec2f {
    var uvs = array<vec2<f32>, 4>(
        vec2<f32>(0., 1.),
        vec2<f32>(0., 0.),
        vec2<f32>(1., 0.),
        vec2<f32>(1., 1.),
    );
    return uvs[corner];
}

@vertex
fn vertex(in: TilemapVertexInput) -> TilemapVertexOutput {
    var out: TilemapVertexOutput;

    let corner = in.v_index % 4u;
    let offset = vert_offset(corner);
    let position_os = vec4f((vec2f(in.tile_index) + offset) * tilemap.tile_render_size, 0., 1.);

    out.position_cs = view.clip_from_view * view.view_from_world * tilemap.world_from_model * position_os;
    out.tint = in.color;

    var atlas_indices: array<u32, 2>;
    if in.atlas_index[2] == 0 {
        atlas_indices = array<u32, 2>(in.atlas_index[0], in.atlas_index[1]);
    } else {
        let fps = f32(animations[in.atlas_index[0] - 1]);
        let offset = f32(in.atlas_index[3]) * 0.001;
        let cur_frame = u32((globals.time + offset) * fps) % in.atlas_index[1];
        let cur_index = cur_frame * 2u + in.atlas_index[0];
        atlas_indices = array<u32, 2>(animations[cur_index], animations[cur_index + 1u]);
    }

    var vert_uv = vert_uv(corner);
    let tile_count = texture_desc[atlas_indices[0]].tile_count;
    let decoded_atlas_index = decode_atlas_and_flip_uv(atlas_indices[1], &vert_uv);
    let atlas_index_2d = vec2u(decoded_atlas_index % tile_count.x, decoded_atlas_index / tile_count.x);
    let tile_uv = vec2f(atlas_index_2d) / vec2f(tile_count);
    
    // out.uv = tile_uv + vert_uv / vec2f(tile_count);
    out.uv = vert_uv;
    // out.uv = vec2f(f32(corner) / 4.);
    out.texture_index = atlas_indices[0];

    return out;
}

@fragment
fn fragment(in: TilemapVertexOutput) -> @location(0) vec4f {
    // return textureSample(texture, texture_sampler, in.uv, in.texture_index) * tilemap.tint;
    return vec4f(in.uv, 0., 1.);
    // return vec4f(1.);
}
