/// The density of stars are related to the distance to center of galaxy.
///
/// Where `x` is the square radius of galaxy.
pub fn star_pos_distr(x: f64) -> impl Fn(f64) -> f64 {
    move |sqr_d| 1. - (sqr_d / x).powi(5)
}

/// In units of solar mass.
pub fn star_mass_pdf(x: f64) -> f64 {
    (1. / (500. * x)).min(1.)
}

/// In units of earth mass.
pub fn planet_mass_pdf(x: f64) -> f64 {
    2. * (-(150. * x).powi(5) + 5.).max(0.) + ((300. * x - 20.).tanh() - (3. * x - 1.).tanh()) * 0.2
}

pub fn moon_mass_pdf(x: f64) -> f64 {
    26f64.powf(-50. * x) + 21f64.powf(-1.5 * (x + 1.))
}

pub fn max_num_planets(star_class_index: u32) -> u32 {
    let x = star_class_index as f32;
    (8. / (1. + ((x - 33.) / 12.).exp()) + 3.).floor() as u32
}

/// In earth mass.
pub fn max_num_moons(mass: f64) -> u32 {
    ((0.5 * mass).sqrt() / 3. + 0.9) as u32
}
