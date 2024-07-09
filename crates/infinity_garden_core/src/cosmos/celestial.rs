use serde::{Deserialize, Serialize};

/// Marker struct for stars.
#[derive(Debug)]
pub struct Star;

/// All celestial dynamic data for a body. Only for calculating physical stuff,
/// like the force between bodies.
pub struct CelestialBodyData {
    pub mass: f64,
    pub radius: f64,
    pub luminosity: f64,
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
