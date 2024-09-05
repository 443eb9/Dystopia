use bevy::math::{IVec2, IVec3};

const H_OFFSETS_FROM_UP: [IVec3; 2] = [IVec3::NEG_Z, IVec3::X];
const H_OFFSETS_FROM_DN: [IVec3; 2] = [IVec3::X, IVec3::NEG_Z];
const V_OFFSET_FROM_UP: IVec3 = IVec3 { x: 0, y: 1, z: -1 };
const V_OFFSET_FROM_DN: IVec3 = IVec3 { x: -1, y: 1, z: 0 };

/// ```text
///         /\/\/\/\/\
///         \/\/\/\/\/\
/// origin-> \/\/\/\/\/
/// ```
///
/// Origin is the left bottom triangle.
pub fn icosahedron(subdivision: u32, ico_origin: IVec3) -> Vec<IVec3> {
    assert_eq!(
        ico_origin.element_sum(),
        1,
        "The direction of triangle at origin should face downwards."
    );

    let level = 2u32.pow(subdivision);
    let mut shape = Vec::with_capacity((20 * 4u32.pow(subdivision)) as usize);

    // Assuming the origin is pointinh downwards.

    // The following comments are based on the assumption that the icosahedron
    // has not been subdivided.

    // The 5 triangles on bottom
    let first_tri = ico_origin;
    for i in 0..5 {
        let origin = first_tri + i * (H_OFFSETS_FROM_DN[0] + H_OFFSETS_FROM_DN[1]) * level as i32;
        subdivided_triangle(&mut shape, level, origin, false);
    }

    // The 10 triangles in the center.
    let layer_length = 5 + 5 * (2u32.pow(subdivision + 1) - 1);
    for layer in 0..level {
        let layer_origin = ico_origin + (layer + level) as i32 * V_OFFSET_FROM_DN;
        let mut cur_triangle = layer_origin;

        for d in 0..layer_length {
            shape.push(cur_triangle);
            cur_triangle += H_OFFSETS_FROM_DN[(d % 2) as usize];
        }
    }

    // The 5 triangles on top
    let first_tri = ico_origin
        + (level as i32 * 2 - 1) * V_OFFSET_FROM_DN
        + IVec3::Z
        + level as i32 * V_OFFSET_FROM_UP;
    for i in 0..5 {
        let origin = first_tri + i * (H_OFFSETS_FROM_UP[0] + H_OFFSETS_FROM_UP[1]) * level as i32;
        subdivided_triangle(&mut shape, level, origin, true);
    }

    shape
}

fn subdivided_triangle(shape: &mut Vec<IVec3>, layers: u32, origin: IVec3, is_up: bool) {
    let h_offsets = if is_up {
        H_OFFSETS_FROM_UP
    } else {
        H_OFFSETS_FROM_DN
    };
    let v_offset = if is_up {
        -V_OFFSET_FROM_UP
    } else {
        V_OFFSET_FROM_DN
    };

    for layer in 0..layers {
        let mut cur_triangle = origin + v_offset * layer as i32;
        let tri_count = 2 * layer + 1;

        for t in 0..tri_count {
            shape.push(cur_triangle);
            cur_triangle += h_offsets[(t % 2) as usize];
        }
    }
}

pub fn rectangle(width: u32, height: u32) -> Vec<IVec2> {
    (0..width as i32)
        .into_iter()
        .flat_map(move |x| (0..height as i32).into_iter().map(move |y| IVec2 { x, y }))
        .collect()
}
