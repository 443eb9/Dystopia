use bevy::math::UVec2;

pub fn rectangle(width: u32, height: u32) -> Vec<UVec2> {
    (0..width)
        .flat_map(move |x| (0..height).map(move |y| UVec2 { x, y }))
        .collect()
}
