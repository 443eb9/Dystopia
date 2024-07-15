//! Physical laws and constants used in the game. These are scaled so don't use them
//! in your homework :p

use std::f64::consts::{PI, TAU};

pub const G: f64 = 6.67430e-11;
pub const STEFAN_BOLTZMANN: f64 = 5.670374419e-8;

pub fn planet_temp_at_dist(star_luminosity: f64, distance: f64, albedo: f64) -> f64 {
    (star_luminosity * (1. - albedo) / (16. * PI * distance * distance * STEFAN_BOLTZMANN))
        .powf(0.25)
}

pub fn planet_dist_when_temp(star_luminosity: f64, temp: f64, albedo: f64) -> f64 {
    ((star_luminosity * (1. - albedo)) / (16. * PI * temp * temp * temp * temp * STEFAN_BOLTZMANN))
        .sqrt()
}

pub fn force_between_at_dist(mass_0: f64, mass_1: f64, distance: f64) -> f64 {
    G * mass_0 * mass_1 / (distance * distance)
}

pub fn dist_when_force_between(mass_0: f64, mass_1: f64, force: f64) -> f64 {
    (G * mass_0 * mass_1 / force).sqrt()
}

pub fn angular_vel_between(center_mass: f64, distance: f64) -> f64 {
    (G * center_mass / (distance * distance * distance)).sqrt()
}

pub fn dist_when_cycle(center_mass: f64, cycle: f64) -> f64 {
    let w = TAU / cycle;
    (G * center_mass / (w * w)).cbrt()
}

pub fn cycle_at_dist(center_mass: f64, distance: f64) -> f64 {
    TAU / angular_vel_between(center_mass, distance)
}
