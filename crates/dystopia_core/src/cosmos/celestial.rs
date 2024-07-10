use bevy::{
    math::DVec2,
    prelude::{Component, Resource},
};
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct Cosmos {
    pub bodies: Vec<CelestialBodyData>,
}

/// The index of this body in cosmos.
///
/// You can fetch the detailed data using this index.
#[derive(Component, Debug, Default)]
pub struct BodyIndex(pub usize);

/// Marker struct for stars.
#[derive(Component, Debug, Default)]
pub struct Star;

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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StarType {
    O,
    B,
    A,
    F,
    G,
    K,
    M,
}
