use bevy::{
    ecs::query::QueryFilter,
    math::DVec2,
    prelude::{Component, Resource, With},
};
use serde::{Deserialize, Serialize};

#[derive(QueryFilter)]
pub struct IsCelestialBody {
    pub star: With<Star>,
    pub planet: With<Planet>,
    pub moon: With<Moon>,
}

#[derive(Resource)]
pub struct Cosmos {
    pub bodies: Vec<CelestialBodyData>,
    pub orbits: Vec<Orbit>,
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
#[derive(Component, Debug, Default, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Default, Clone, Copy)]
pub struct Orbit {
    pub initial_theta: f64,
    pub center_id: usize,
    pub center: DVec2,
    pub radius: f64,
    pub sidereal_period: u64,
    pub rotation_period: u64,
}
