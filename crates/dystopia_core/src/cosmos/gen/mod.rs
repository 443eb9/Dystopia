use std::{
    f64::consts::{PI, TAU},
    ops::Range,
    time::Instant,
};

use bevy::{
    asset::Assets,
    color::LinearRgba,
    log::info,
    math::{FloatExt, Vec3, VectorSpace},
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
        celestial::{BodyIndex, CelestialBodyData, Cosmos, Orbit},
        config::{CosmosStarPropertiesConfig, StarClass},
        gen::distr::*,
        mesh::{GiantBodyMaterial, RockyBodyMaterial, StarMaterial},
    },
    math::{self, reject_sampling},
    schedule::signal::InitializationSignal,
    sci::{
        physics,
        unit::{Length, Mass, Time, Unit},
    },
    simulation::{GlobalRng, Ticker},
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
    pub body: CelestialBodyData,
    pub orbit: Orbit,
    pub effective_temp: f32,
    pub is_giant: bool,
    pub children: Vec<MoonData>,
}

#[derive(Debug)]
pub struct MoonData {
    pub body: CelestialBodyData,
    pub orbit: Orbit,
    pub effective_temp: f32,
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

    info!("Start generating bodies...");

    let mut rng = StdRng::seed_from_u64(settings.seed);

    let mut stars = generate_stars(&mut rng, &settings, &star_props);
    for (i_star, star) in stars.iter_mut().enumerate() {
        let mut planets = generate_planets(&mut rng, &star, i_star);
        for (i_planet, planet) in planets.iter_mut().enumerate() {
            planet.children = generate_moon(&mut rng, planet, i_planet);
        }
        star.children = planets;
    }

    convert_units(&mut stars);

    info!("Start placing bodies...");

    stars.iter_mut().enumerate().for_each(|(i_star, star)| {
        place_stars(&mut rng, &settings, star);
        place_planets(&mut rng, star, i_star);

        star.children
            .iter_mut()
            .enumerate()
            .for_each(|(i_planet, planet)| place_moons(&mut rng, planet, i_planet));
    });

    info!("Start generating orbits...");

    let orbits = generate_orbits(&mut rng, &mut stars);

    info!("Start spawning all bodies into game...");

    // Circles are implemented in shader, so we only need a square here.
    let mesh = Mesh2dHandle(meshes.add(Rectangle::from_length(1.)));

    let bodies = spawn_bodies(
        &mut commands,
        stars,
        mesh,
        &mut star_materials,
        &mut rocky_body_materials,
        &mut giant_body_materials,
    );

    commands.insert_resource(Cosmos { bodies, orbits });
    commands.insert_resource(GlobalRng(rng));
    commands.insert_resource(Ticker(0));

    info!(
        "Cosmos generation finished after {} s!",
        start.elapsed().as_secs_f32(),
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
        });
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
            star_index, i_planet, n
        );

        let (density, is_giant) = if mass > 100. {
            (rng.sample(giant_density_distr), true)
        } else {
            (rng.sample(rocky_density_distr), true)
        };

        let radius = (Mass::EarthMass(mass).to_si() / density * 0.75 / PI).cbrt();

        planets.push(PlanetData {
            body: CelestialBodyData {
                pos: Default::default(),
                mass,
                radius,
            },
            orbit: Default::default(),
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
            planet_index, i_moon, n
        );

        let radius = (Mass::EarthMass(mass).to_si() / density * 0.75 / PI).cbrt();

        moons.push(MoonData {
            body: CelestialBodyData {
                pos: Default::default(),
                mass,
                radius,
            },
            orbit: Default::default(),
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

    star.body.pos = math::polar_to_cartesian(theta, r);
}

fn place_planets(rng: &mut impl Rng, star: &mut StarData, star_index: usize) {
    let mut cur_planet = 0;

    // --- Planets inside CHZ ---
    // Circumstellar Habitable Zone
    let chz_near = physics::planetary_dist(star.effective_temp, star.body.radius, 400.);
    let chz_far = physics::planetary_dist(star.effective_temp, star.body.radius, 200.);

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
    let farthest = physics::dist_when_force_between(star.body.mass, 1., 1e4);
    // Temperature higher than 2000 K
    let closest = physics::planetary_dist(star.effective_temp, star.body.radius, 2000.);

    let proportion = rng.gen_range(0.2..0.8);
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

    (cur_planet as usize..star.children.len()).for_each(|i| {
        info!("Discarding planet with parent star {}: {}", star_index, i);
        star.children.pop();
    });
}

fn place_moons(rng: &mut impl Rng, planet: &mut PlanetData, planet_index: usize) {
    let closest = planet.body.radius * 1.5;
    let farthest = physics::dist_when_force_between(planet.body.mass, 1., 1e5);

    let succeeded = scatter_bodies_in_range(
        rng,
        &planet.body,
        planet.children.iter_mut().map(|m| &mut m.body).collect(),
        [closest, farthest],
        10,
    );

    (succeeded as usize..planet.children.len()).for_each(|i| {
        info!("Discarding moon with parent planet {}: {}", planet_index, i);
        planet.children.pop();
    });
}

/// Returns the number of bodies successfully scattered.
fn scatter_bodies_in_range(
    rng: &mut impl Rng,
    center: &CelestialBodyData,
    mut bodies: Vec<&mut CelestialBodyData>,
    boundaries: [f64; 2],
    max_failed: u32,
) -> u32 {
    let mut cur_dist = boundaries[0];
    let mut cur_body = 0;
    let mut cur_failed = 0;

    while cur_body < bodies.len() {
        let t = rng.gen_range(0f64..1f64 / (bodies.len() as f64 * 0.8));
        let delta = (boundaries[1] - boundaries[0]) * t;
        if cur_dist + delta < boundaries[1] {
            bodies[cur_body].pos =
                center.pos + math::polar_to_cartesian(rng.gen_range(0f64..TAU), cur_dist);

            cur_body += 1;
            cur_dist += delta;
        } else {
            cur_failed += 1;
        }

        if cur_failed >= max_failed {
            break;
        }
    }

    cur_body as u32
}

fn generate_orbits(rng: &mut impl Rng, stars: &mut Vec<StarData>) -> Vec<Orbit> {
    let mut orbits = Vec::with_capacity(stars.len() * 5);
    let mut cur_body = 0;

    for star in stars {
        let i_star = cur_body;
        orbits.push(Orbit {
            radius: -1.,
            ..Default::default()
        });

        for planet in &mut star.children {
            let distance = (planet.body.pos - star.body.pos).length();
            let i_planet = cur_body;
            orbits.push(Orbit {
                initial_theta: rng.gen_range(0f64..TAU),
                center_id: i_star,
                center: Default::default(),
                radius: distance,
                sidereal_period: Time::Second(
                    (TAU / physics::angular_vel_between(star.body.mass, distance)) as u64,
                )
                .to_si(),
                rotation_period: Time::Second(rng.gen_range(100..1800)).to_si(),
            });
            cur_body += 1;

            for moon in &mut planet.children {
                let distance = (moon.body.pos - planet.body.pos).length();
                orbits.push(Orbit {
                    initial_theta: rng.gen_range(0f64..TAU),
                    center_id: i_planet,
                    center: Default::default(),
                    radius: distance,
                    sidereal_period: Time::Second(
                        (TAU / physics::angular_vel_between(planet.body.mass, distance)) as u64,
                    )
                    .to_si(),
                    rotation_period: Time::Second(rng.gen_range(100..1800)).to_si(),
                });
                cur_body += 1;
            }
        }
    }

    dbg!(orbits)
}

fn spawn_bodies(
    commands: &mut Commands,
    stars: Vec<StarData>,
    mesh: Mesh2dHandle,
    star_materials: &mut Assets<StarMaterial>,
    rocky_body_materials: &mut Assets<RockyBodyMaterial>,
    giant_body_materials: &mut Assets<GiantBodyMaterial>,
) -> Vec<CelestialBodyData> {
    // Estimate the number of bodies roughly equal to star.len() * 5
    let mut bodies = Vec::with_capacity(stars.len() * 5);

    for star in stars {
        commands.spawn(StarBundle {
            star_ty: star.class.ty,
            body_index: BodyIndex(bodies.len()),
            mesh: mesh.clone(),
            material: star_materials.add(StarMaterial { color: star.color }),
            transform: Transform::from_scale(Vec3::splat(
                Length::SolarRadius(star.body.radius * 2.).to_si() as f32,
            )),
            ..Default::default()
        });
        bodies.push(star.body);

        for planet in star.children {
            if planet.is_giant {
                commands.spawn(GiantBodyBundle {
                    body_index: BodyIndex(bodies.len()),
                    mesh: mesh.clone(),
                    material: giant_body_materials.add(GiantBodyMaterial {
                        // TODO: Generate color
                        color: LinearRgba::WHITE,
                    }),
                    transform: Transform::from_scale(Vec3::splat(
                        Length::RenderMeter(planet.body.radius * 2.).to_si() as f32,
                    )),
                    ..Default::default()
                });
            } else {
                commands.spawn(RockyBodyBundle {
                    body_index: BodyIndex(bodies.len()),
                    mesh: mesh.clone(),
                    material: rocky_body_materials.add(RockyBodyMaterial {
                        // TODO: Generate color
                        color: LinearRgba::WHITE,
                    }),
                    transform: Transform::from_scale(Vec3::splat(
                        Length::RenderMeter(planet.body.radius * 2.).to_si() as f32,
                    )),
                    ..Default::default()
                });
            }

            bodies.push(planet.body);

            for moon in planet.children {
                commands.spawn(RockyBodyBundle {
                    body_index: BodyIndex(bodies.len()),
                    mesh: mesh.clone(),
                    material: rocky_body_materials.add(RockyBodyMaterial {
                        // TODO: Generate color
                        color: LinearRgba::WHITE,
                    }),
                    transform: Transform::from_scale(Vec3::splat(
                        Length::RenderMeter(moon.body.radius * 2.).to_si() as f32,
                    )),
                    ..Default::default()
                });

                bodies.push(moon.body);
            }
        }
    }

    bodies
}
