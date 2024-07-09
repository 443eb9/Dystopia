use std::{f64::consts::PI, ops::Range};

use bevy::{
    log::info,
    math::{FloatExt, VectorSpace},
    prelude::{Commands, Res, ResMut, Resource},
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::Normal;

use crate::{
    cosmos::{
        celestial::CelestialBodyData,
        config::{CosmosStarPropertiesConfig, StarProperties},
        gen::distr::*,
    },
    math::reject_sampling,
    schedule::signal::InitializationSignal,
};

mod distr;

#[derive(Debug)]
pub struct StarData {
    pub props: StarProperties,
    pub children: Vec<PlanetData>,
}

#[derive(Debug)]
pub struct PlanetData {
    pub parent: usize,
    pub body: CelestialBodyData,
    pub children: Vec<MoonData>,
}

#[derive(Debug)]
pub struct MoonData {
    pub parent: usize,
    pub body: CelestialBodyData,
}

#[derive(Resource)]
pub struct CosmosGenerationSettings {
    pub seed: u64,
    pub num_stars: Range<u32>,
}

pub fn generate_cosmos(
    mut commands: Commands,
    mut signal: ResMut<InitializationSignal>,
    star_props: Res<CosmosStarPropertiesConfig>,
    settings: Res<CosmosGenerationSettings>,
) {
    if signal.cosmos_initialized {
        return;
    }

    info!("Start generating cosmos...");

    let mut rng = StdRng::seed_from_u64(settings.seed);

    info!("Generating basic celestial bodies...");

    let mut num_planets = 0;
    let mut num_moons = 0;

    let mut stars = generate_stars(&mut rng, &settings, &star_props);
    for (i_star, star) in stars.iter_mut().enumerate() {
        let mut planets = generate_planets(&mut rng, &star, i_star);
        num_planets += planets.len();
        for (i_planet, planet) in planets.iter_mut().enumerate() {
            planet.children = generate_moon(&mut rng, planet, i_planet);
            num_moons += planet.children.len();
        }
        star.children = planets;
    }

    info!(
        "Successfully generated celestial bodies: stars: {}, planets: {}, moons: {}",
        stars.len(),
        num_planets,
        num_moons
    );

    signal.cosmos_initialized = true;
}

fn generate_stars(
    rng: &mut impl Rng,
    settings: &CosmosGenerationSettings,
    star_props: &CosmosStarPropertiesConfig,
) -> Vec<StarData> {
    let n = rng.gen_range(settings.num_stars.clone());
    let masses = reject_sampling(rng, star_mass_pdf, 0f64..130f64, 0f64..1f64, n, n * 2);

    let star_props = star_props.get();
    let mut stars = Vec::with_capacity(n as usize);

    // Unit: Solar mass
    for mass in masses {
        let floor = star_props
            .iter()
            .enumerate()
            .find_map(|(i, p)| if p.mass < mass { Some(i) } else { None })
            .unwrap_or_else(|| star_props.len() - 1);
        let ceil = floor.saturating_sub(1);

        let (floor, ceil) = (&star_props[floor], &star_props[ceil]);

        let props = StarProperties {
            class: floor.class,
            mass: floor.mass.lerp(ceil.mass, rng.gen_range(0f64..1f64)),
            radius: floor.radius.lerp(ceil.radius, rng.gen_range(0f64..1f64)),
            luminosity: floor
                .luminosity
                .lerp(ceil.luminosity, rng.gen_range(0f64..1f64)),
            effective_temp: floor
                .effective_temp
                .lerp(ceil.effective_temp, rng.gen_range(0f64..1f64)),
            color: floor.color.lerp(ceil.color, rng.gen_range(0f32..1f32)),
        };

        stars.push(StarData {
            props,
            children: Vec::new(),
        })
    }

    stars
}

fn generate_planets(rng: &mut impl Rng, star: &StarData, star_index: usize) -> Vec<PlanetData> {
    let n = rng.gen_range(0..=max_num_planets(star.props.class));

    let masses = reject_sampling(
        rng,
        planet_mass_pdf,
        0.02f64..300f64,
        0f64..9.952f64,
        n,
        n * 2,
    );

    let rocky_density_distr = Normal::<f64>::new(0.5, 0.25).unwrap();
    let giant_density_distr = Normal::<f64>::new(0.5, 0.25).unwrap();

    let mut planets = Vec::with_capacity(n as usize);

    // Unit: Earth mass
    for mass in masses {
        let density = if mass > 100. {
            rng.sample(giant_density_distr)
        } else {
            rng.sample(rocky_density_distr)
        };

        let radius = (mass / density * 0.75 / PI).cbrt();

        planets.push(PlanetData {
            parent: star_index,
            body: CelestialBodyData { mass, radius },
            children: Vec::new(),
        });
    }

    planets
}

fn generate_moon(rng: &mut impl Rng, planet: &PlanetData, planet_index: usize) -> Vec<MoonData> {
    let n = rng.gen_range(0..=max_num_moons(planet.body.mass));

    let masses = reject_sampling(rng, moon_mass_pdf, 0f64..1f64, 0f64..1f64, n, n * 2);
    let density = (0..n).map(|_| rng.sample(Normal::<f64>::new(0.5, 0.25).unwrap()));

    let mut moons = Vec::with_capacity(n as usize);

    for (mass, density) in masses.into_iter().zip(density) {
        let radius = (mass / density * 0.75 / PI).cbrt();

        moons.push(MoonData {
            parent: planet_index,
            body: CelestialBodyData { mass, radius },
        });
    }

    moons
}
