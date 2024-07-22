#import bevy_render::{view::View, globals::Globals}

struct Tilemap {
    edge_length: f32,
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
    @location(3) tile_index: vec3i,
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

const SQRT3 = sqrt(3.);

// Adapted from https://github.com/BorisTheBrave/grids/blob/main/src/updown_tri.py
fn tri_center(index: vec3i, edge_length: f32) -> vec2f {
    let tri = vec3f(index);
    return vec2f(
        (0.5 * tri.x + -0.5 * tri.z) * edge_length,
        (-SQRT3 / 6. * tri.x + SQRT3 / 3. * tri.y - SQRT3 / 6. * tri.z) * edge_length,
    );
}

fn tri_is_up(index: vec3i) -> bool {
    return index.x + index.y + index.z == 2;
}

fn tri_corner(index: vec3i, edge_length: f32, corner: u32) -> vec2f {
    var d = vec3i(0);
    d[corner] = 1;

    if tri_is_up(index) {
        return tri_center(index + d, edge_length);
    } else {
        return tri_center(index - d, edge_length);
    }
}

var<private> UVS_UP: array<vec2f, 3> = array<vec2f, 3>(
    vec2f(1., 1.),
    vec2f(0.5, 0.),
    vec2f(0., 1.),
);

var<private> UVS_DN: array<vec2f, 3> = array<vec2f, 3>(
    vec2f(0., 0.),
    vec2f(0.5, 1.),
    vec2f(1., 0.),
);

fn tri_uv(index: vec3i, corner: u32) -> vec2f {
    if tri_is_up(index) {
        return UVS_UP[corner];
    } else {
        return UVS_DN[corner];
    }
}

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

@vertex
fn vertex(in: TilemapVertexInput) -> TilemapVertexOutput {
    var out: TilemapVertexOutput;

    let corner = in.v_index % 3u;
    let position_os = vec4f(tri_corner(in.tile_index, tilemap.edge_length, corner), 0., 1.);

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

    var vert_uv = tri_uv(in.tile_index, corner);
    let tile_count = texture_desc[atlas_indices[0]].tile_count;
    let decoded_atlas_index = decode_atlas_and_flip_uv(atlas_indices[1], &vert_uv);
    let atlas_index_2d = vec2u(decoded_atlas_index % tile_count.x, decoded_atlas_index / tile_count.x);
    let tile_uv = vec2f(atlas_index_2d) / vec2f(tile_count);
    out.uv = tile_uv + vert_uv / vec2f(tile_count);
    out.texture_index = atlas_indices[0];

    return out;
}

@fragment
fn fragment(in: TilemapVertexOutput) -> @location(0) vec4f {
    return textureSample(texture, texture_sampler, in.uv, in.texture_index);
}
