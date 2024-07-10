use std::f64::consts::PI;

pub const G: f64 = 6.67430e-11;
pub const STEFAN_BOLTZMANN: f64 = 5.670374419e-8;

pub fn planetary_temp(star_effective_temp: f64, star_radius: f64, distance: f64) -> f64 {
    star_effective_temp * (star_radius / (2. * distance)).sqrt()
}

pub fn planetary_dist(
    star_effective_temp: f64,
    star_radius: f64,
    planet_effective_temp: f64,
) -> f64 {
    (star_effective_temp * star_effective_temp * star_radius)
        / (2. * planet_effective_temp * planet_effective_temp)
}

pub fn planetary_flux(star_effective_temp: f64, distance: f64) -> f64 {
    star_effective_temp / (4. * PI * distance * distance)
}

pub fn force_between_at_dist(mass_0: f64, mass_1: f64, distance: f64) -> f64 {
    G * mass_0 * mass_1 / (distance * distance)
}

pub fn dist_when_force_between(mass_0: f64, mass_1: f64, force: f64) -> f64 {
    (G * mass_0 * mass_1 / force).sqrt()
}
