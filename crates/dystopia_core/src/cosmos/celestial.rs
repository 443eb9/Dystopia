use bevy::{
    color::LinearRgba,
    math::DVec2,
    prelude::{Component, Deref, DerefMut, Entity, Resource},
};
use serde::{Deserialize, Serialize};

use crate::tuple_struct_new;

#[derive(Resource)]
pub struct OrbitsVisibility {
    pub scale_threshold: f32,
    pub alpha: f32,
    pub fade_speed: f32,
}

impl Default for OrbitsVisibility {
    fn default() -> Self {
        Self {
            scale_threshold: 1.,
            alpha: 0.7,
            fade_speed: 20.,
        }
    }
}

#[derive(Resource)]
pub struct Cosmos {
    pub bodies: Vec<CelestialBodyData>,
    pub entities: Vec<Entity>,
    pub orbits: Vec<Orbit>,
}

/// The index of this body in cosmos.
///
/// You can fetch the detailed data using this index.
#[derive(Component, Debug, Default, Clone, Copy, Deref)]
pub struct BodyIndex(usize);
tuple_struct_new!(BodyIndex, usize);

/// The index of this orbit in cosmos.
///
/// You can fetch the detailed data using this index.
#[derive(Component, Debug, Default, Clone, Copy, Deref)]
pub struct OrbitIndex(usize);
tuple_struct_new!(OrbitIndex, usize);

/// Marker struct for stars.
#[derive(Component, Debug, Default)]
pub struct Star;

/// All bodies in this system.
#[derive(Component, Default, Deref, DerefMut)]
pub struct System(Vec<BodyIndex>);
tuple_struct_new!(System, Vec<BodyIndex>);

/// Marker struct for planets.
#[derive(Component, Debug, Default)]
pub struct Planet;

// Marker struct for moons.
#[derive(Component, Debug, Default)]
pub struct Moon;

/// All celestial dynamic data for a body.
#[derive(Debug)]
pub struct CelestialBodyData {
    pub pos: DVec2,
    pub mass: f64,
    pub radius: f64,
}

/// The type of a main sequence star.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StarType {
    #[default]
    O,
    B,
    A,
    F,
    G,
    K,
    M,
}

/// Detailed class of a main sequence star.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StarClass {
    pub ty: StarType,
    pub sub_ty: u32,
    pub index: u32,
}

/// The type of a main sequence star.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyType {
    #[default]
    Rocky,
    GasGiant,
    IceGiant,
}

/// Mark a body as landable, which means players can land on that body and
/// build facilities.
///
/// Stars, gas/ice giants and small asteroids are generally not landable.
// TODO maybe support landing on these bodies after some technologies.
#[derive(Component)]
pub struct Landable;

/// The color of the body. Should keep synced with mesh color and orbit color.
#[derive(Component, Default, Clone, Deref, DerefMut)]
pub struct BodyColor(LinearRgba);
tuple_struct_new!(BodyColor, LinearRgba);

/// The temperature of the body, in Kelvin.
#[derive(Component, Default, Clone, Deref, DerefMut)]
pub struct BodyTemperature(f64);
tuple_struct_new!(BodyTemperature, f64);

/// The luminous intensity of the body received from its parent star.
#[derive(Component, Default, Clone, Deref, DerefMut)]
pub struct BodyIlluminance(f64);
tuple_struct_new!(BodyIlluminance, f64);

/// The corresponding tilemap to the body.
///
/// This won't be added to the body when they're spawned, as it will cause too much
/// performance overhead.
#[derive(Component, Deref, Clone, Copy)]
pub struct BodyTilemap(Entity);
tuple_struct_new!(BodyTilemap, Entity);

/// Marks a body needs to generate/load tilemap. The generation process will happen
/// asynchronously.
#[derive(Component)]
pub struct ToLoadTilemap;

/// Marks a body needs to save the tilemap onto disk.
#[derive(Component)]
pub struct ToSaveTilemap {
    /// Whether to remove tilemap from the entity.
    pub remove_after_done: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Orbit {
    pub initial_progress: f64,
    pub center_id: usize,
    pub center: DVec2,
    pub radius: f64,
    pub sidereal_period: u64,
    pub rotation_period: u64,
}
