use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

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
