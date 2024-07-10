use std::{
    f64::consts::{PI, TAU},
    ops::Range,
    time::Instant,
};

use bevy::{
    asset::Assets,
    color::LinearRgba,
    log::info,
    math::{DVec2, FloatExt, Vec3, VectorSpace},
    prelude::{Commands, Rectangle, Res, ResMut, Resource},
    render::mesh::Mesh,
    sprite::Mesh2dHandle,
    transform::components::Transform,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::Normal;

use crate::{
    cosmos::{
        bundle::{GiantBodyBundle, RockyBodyBundle, StarBundle},
        celestial::{BodyIndex, CelestialBodyData, Cosmos},
        config::{CosmosStarPropertiesConfig, StarClass},
        gen::distr::*,
        mesh::{GiantBodyMaterial, RockyBodyMaterial, StarMaterial},
    },
    math::{self, reject_sampling},
    schedule::signal::InitializationSignal,
    sci::{
        physics,
        unit::{Length, Mass, Unit},
    },
    simulation::GlobalRng,
};

mod distr;

#[derive(Debug)]
pub struct StarData {
    pub body: CelestialBodyData,
    pub class: StarClass,
    pub effective_temp: f64,
    pub color: LinearRgba,
    pub children: Vec<PlanetData>,
}

#[derive(Debug)]
pub struct PlanetData {
    pub parent: usize,
    pub body: CelestialBodyData,
    pub effective_temp: f32,
    pub is_giant: bool,
    pub children: Vec<MoonData>,
}

#[derive(Debug)]
pub struct MoonData {
    pub parent: usize,
    pub effective_temp: f32,
    pub body: CelestialBodyData,
}

#[derive(Resource)]
pub struct CosmosGenerationSettings {
    pub seed: u64,
    pub galaxy_radius: f64,
    pub num_stars: Range<u32>,
}

pub fn generate_cosmos(
    mut commands: Commands,
    mut signal: ResMut<InitializationSignal>,
    star_props: Res<CosmosStarPropertiesConfig>,
    settings: Res<CosmosGenerationSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut star_materials: ResMut<Assets<StarMaterial>>,
    mut rocky_body_materials: ResMut<Assets<RockyBodyMaterial>>,
    mut giant_body_materials: ResMut<Assets<GiantBodyMaterial>>,
) {
    if signal.cosmos_initialized {
        return;
    }

    signal.cosmos_initialized = true;
    info!("Start generating cosmos...");

    let start = Instant::now();

    info!("Generating basic celestial bodies...");

    let mut rng = StdRng::seed_from_u64(settings.seed);
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

    convert_units(&mut stars);

    info!(
        "Successfully generated basic celestial bodies! stars: {}, planets: {}, moons: {}",
        stars.len(),
        num_planets,
        num_moons
    );
    info!("Start placing bodies...");

    stars.iter_mut().for_each(|star| {
        place_stars(&mut rng, &settings, star);
        place_planets(&mut rng, star);

        star.children
            .iter_mut()
            .for_each(|planet| place_moons(&mut rng, planet));
    });

    info!("Successfully placed all bodies!");
    info!("Start spawning all bodies into game...");

    // Circles are implemented in shader, so we only need a square here.
    let mesh = Mesh2dHandle(meshes.add(Rectangle::from_length(1.)));

    spawn_bodies(
        &mut commands,
        stars,
        mesh,
        &mut star_materials,
        &mut rocky_body_materials,
        &mut giant_body_materials,
    );

    info!("Successfully spawned all bodies!");

    commands.insert_resource(GlobalRng(rng));

    info!(
        "Cosmos generation finished after {} s!",
        start.elapsed().as_secs_f32()
    );
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
    for (i_star, mass) in masses.into_iter().enumerate() {
        info!("Generating stars {}/{}", i_star, n);

        let floor = star_props
            .iter()
            .enumerate()
            .find_map(|(i, p)| if p.mass < mass { Some(i) } else { None })
            .unwrap_or_else(|| star_props.len() - 1);
        let ceil = floor.saturating_sub(1);

        let (floor, ceil) = (&star_props[floor], &star_props[ceil]);
        let radius = floor.radius.lerp(ceil.radius, rng.gen_range(0f64..1f64));
        let effective_temp = floor
            .effective_temp
            .lerp(ceil.effective_temp, rng.gen_range(0f64..1f64));

        stars.push(StarData {
            body: CelestialBodyData {
                pos: Default::default(),
                mass,
                radius,
            },
            class: floor.class,
            effective_temp,
            color: floor.color.lerp(ceil.color, rng.gen_range(0f32..1f32)),
            children: Vec::new(),
        })
    }

    stars
}

fn generate_planets(rng: &mut impl Rng, star: &StarData, star_index: usize) -> Vec<PlanetData> {
    let n = rng.gen_range(0..=max_num_planets(star.class));

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
    for (i_planet, mass) in masses.into_iter().enumerate() {
        info!(
            "Generating planets with parent star {}: {}/{}",
            star_index,
            i_planet + 1,
            n
        );

        let (density, is_giant) = if mass > 100. {
            (rng.sample(giant_density_distr), true)
        } else {
            (rng.sample(rocky_density_distr), true)
        };

        let radius = (Mass::EarthMass(mass).to_si() / density * 0.75 / PI).cbrt();

        planets.push(PlanetData {
            parent: star_index,
            body: CelestialBodyData {
                pos: Default::default(),
                mass,
                radius,
            },
            effective_temp: 0.,
            is_giant,
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

    // Unit: Earth mass
    for (i_moon, (mass, density)) in masses.into_iter().zip(density).enumerate() {
        info!(
            "Generating moon with parent planet {}: {}/{}",
            planet_index,
            i_moon + 1,
            n
        );

        let radius = (Mass::EarthMass(mass).to_si() / density * 0.75 / PI).cbrt();

        moons.push(MoonData {
            parent: planet_index,
            body: CelestialBodyData {
                pos: Default::default(),
                mass,
                radius,
            },
            effective_temp: 0.,
        });
    }

    moons
}

fn convert_units(stars: &mut Vec<StarData>) {
    for star in stars {
        star.body.mass = Mass::SolarMass(star.body.mass).to_si();
        star.body.radius = Length::SolarRadius(star.body.radius).to_si();

        for planet in &mut star.children {
            planet.body.mass = Mass::EarthMass(planet.body.mass).to_si();

            for moon in &mut planet.children {
                moon.body.mass = Mass::EarthMass(moon.body.mass).to_si();
            }
        }
    }
}

fn place_stars(rng: &mut impl Rng, settings: &CosmosGenerationSettings, star: &mut StarData) {
    let theta = rng.gen_range(0f64..TAU);
    let r = reject_sampling(
        rng,
        star_pos_distr(settings.galaxy_radius),
        0f64..settings.galaxy_radius * settings.galaxy_radius,
        0f64..1f64,
        1,
        2,
    )[0];

    star.body.pos = DVec2::from(math::polar_to_cartesian(theta, r));
}

fn place_planets(rng: &mut impl Rng, star: &mut StarData) {
    let mut cur_planet = 0;

    // --- Planets inside CHZ ---
    // Circumstellar Habitable Zone
    let chz_near = physics::planetary_dist(star.effective_temp, star.body.radius, 400.);
    let chz_far = physics::planetary_dist(star.effective_temp, star.body.radius, 200.);

    dbg!(chz_near, chz_far);

    let chz_n = rng.sample(Normal::<f64>::new(0., 0.6).unwrap()).floor() as usize;

    cur_planet += scatter_bodies_in_range(
        rng,
        &star.body,
        star.children
            .iter_mut()
            .take(chz_n)
            .map(|p| &mut p.body)
            .collect(),
        [chz_near, chz_far],
        10,
    );

    // Calculate boundaries
    // Acceleration below 1e6 m/s^2
    let farthest = physics::dist_when_force_between(star.body.mass, 1., 1e6);
    // Temperature higher than 2000 K
    let closest = physics::planetary_dist(star.effective_temp, star.body.radius, 2000.);

    let proportion = (chz_near - closest) / (farthest - chz_far);
    let remaining_planets = star.children.len() as u32 - cur_planet;

    // --- Planets too close to star ---

    let n_too_close = (proportion * remaining_planets as f64).round() as u32;

    cur_planet += scatter_bodies_in_range(
        rng,
        &star.body,
        star.children
            .iter_mut()
            .skip(cur_planet as usize)
            .take(n_too_close as usize)
            .map(|p| &mut p.body)
            .collect(),
        [closest, chz_near],
        10,
    );

    // --- Planets too faraway from star ---

    let n_too_far = remaining_planets - n_too_close;

    cur_planet += scatter_bodies_in_range(
        rng,
        &star.body,
        star.children
            .iter_mut()
            .skip(cur_planet as usize)
            .take(n_too_far as usize)
            .map(|p| &mut p.body)
            .collect(),
        [chz_far, farthest],
        10,
    );

    // Remove planets that are too far

    (cur_planet as usize..star.children.len()).for_each(|_| {
        star.children.pop();
    });
}

fn place_moons(rng: &mut impl Rng, planet: &mut PlanetData) {
    let closest = planet.body.radius * 1.5;
    let farthest = physics::dist_when_force_between(planet.body.mass, 1., 1e5);

    let succeeded = scatter_bodies_in_range(
        rng,
        &planet.body,
        planet.children.iter_mut().map(|m| &mut m.body).collect(),
        [closest, farthest],
        10,
    );

    (succeeded as usize..planet.children.len()).for_each(|_| {
        planet.children.pop();
    });
}

/// Returns the number of bodies successfully scattered.
fn scatter_bodies_in_range(
    rng: &mut impl Rng,
    center: &CelestialBodyData,
    mut bodies: Vec<&mut CelestialBodyData>,
    boundaies: [f64; 2],
    max_failed: u32,
) -> u32 {
    let mut cur_dist = boundaies[0];
    let mut cur_body = 0;
    let mut cur_failed = 0;

    while cur_body < bodies.len() {
        let t = rng.gen_range(0f64..1f64);
        cur_dist += boundaies[1] - boundaies[0] * t;
        if cur_dist < boundaies[1] {
            bodies[cur_body].pos = center.pos
                + DVec2::from(math::polar_to_cartesian(rng.gen_range(0f64..TAU), cur_dist));

            cur_body += 1;
        } else {
            cur_failed += 1;
        }

        if cur_failed >= max_failed {
            break;
        }
    }

    cur_body as u32
}

fn spawn_bodies(
    commands: &mut Commands,
    stars: Vec<StarData>,
    mesh: Mesh2dHandle,
    star_materials: &mut Assets<StarMaterial>,
    rocky_body_materials: &mut Assets<RockyBodyMaterial>,
    giant_body_materials: &mut Assets<GiantBodyMaterial>,
) {
    // Estimate the number of bodies roughly equal to star.len() * 5
    let mut bodies = Vec::with_capacity(stars.len() * 5);

    for star in stars {
        commands.spawn(StarBundle {
            body_index: BodyIndex(bodies.len()),
            mesh: mesh.clone(),
            material: star_materials.add(StarMaterial { color: star.color }),
            transform: Transform::from_scale(Vec3::splat(star.body.radius as f32 * 2.)),
            ..Default::default()
        });
        bodies.push(star.body);

        for planet in star.children {
            if planet.is_giant {
                commands.spawn(GiantBodyBundle {
                    body_index: BodyIndex(bodies.len()),
                    mesh: mesh.clone(),
                    material: giant_body_materials.add(GiantBodyMaterial {
                        // TODO Generate color
                        color: LinearRgba::WHITE,
                    }),
                    transform: Transform::from_scale(Vec3::splat(planet.body.radius as f32 * 2.)),
                    ..Default::default()
                });
            } else {
                commands.spawn(RockyBodyBundle {
                    body_index: BodyIndex(bodies.len()),
                    mesh: mesh.clone(),
                    material: rocky_body_materials.add(RockyBodyMaterial {
                        // TODO Generate color
                        color: LinearRgba::WHITE,
                    }),
                    transform: Transform::from_scale(Vec3::splat(planet.body.radius as f32 * 2.)),
                    ..Default::default()
                });
            }

            bodies.push(planet.body);

            for moon in planet.children {
                commands.spawn(RockyBodyBundle {
                    body_index: BodyIndex(bodies.len()),
                    mesh: mesh.clone(),
                    material: rocky_body_materials.add(RockyBodyMaterial {
                        // TODO Generate color
                        color: LinearRgba::WHITE,
                    }),
                    transform: Transform::from_scale(Vec3::splat(moon.body.radius as f32 * 2.)),
                    ..Default::default()
                });

                bodies.push(moon.body);
            }
        }
    }

    commands.insert_resource(Cosmos { bodies });
}
