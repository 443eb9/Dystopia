use std::{
    f64::consts::{PI, TAU},
    ops::Range,
    time::Instant,
};

use avian2d::prelude::Collider;
use bevy::{
    asset::{Assets, Handle},
    color::{Alpha, LinearRgba},
    core::Name,
    log::info,
    math::{DVec2, FloatExt, Vec3, VectorSpace},
    prelude::{Commands, Entity, Rectangle, Res, ResMut, Resource},
    render::mesh::Mesh,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    transform::components::Transform,
};
use indexmap::IndexSet;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{num_traits::Float, Distribution, Normal, StandardNormal};

use crate::{
    cosmos::{
        bundle::{GiantBodyBundle, RockyBodyBundle, StarBundle},
        celestial::{
            BodyIndex, BodyType, CelestialBodyData, Cosmos, Moon, Orbit, OrbitIndex, Planet,
            StarClass,
        },
        config::{CosmosStarNamesConfig, CosmosStarPropertiesConfig},
        gen::distr::*,
        mesh::{GiantBodyMaterial, OrbitMaterial, RockyBodyMaterial, StarMaterial},
        ORBIT_MESH_SCALE, ORBIT_WIDTH,
    },
    math::reject_sampling,
    schedule::signal::InitializationSignal,
    sci::{
        physics,
        unit::{Length, Mass, RadiantFlux, Time, Unit},
    },
    simulation::{GlobalRng, Ticker},
};

mod distr;

#[derive(Debug)]
pub struct StarData {
    pub name: String,
    pub body: CelestialBodyData,
    pub class: StarClass,
    pub luminosity: f64,
    pub color: LinearRgba,
    pub children: Vec<PlanetData>,
}

#[derive(Debug)]
pub struct PlanetData {
    pub body: CelestialBodyData,
    pub ty: BodyType,
    pub orbit: Orbit,
    pub effective_temp: f32,
    pub color: LinearRgba,
    pub children: Vec<MoonData>,
}

#[derive(Debug)]
pub struct MoonData {
    pub body: CelestialBodyData,
    pub orbit: Orbit,
    pub effective_temp: f32,
    pub color: LinearRgba,
}

#[derive(Debug, Default)]
pub struct CosmosBodiesStatistics {
    pub num_stars: u32,
    pub num_planets: u32,
    pub num_moons: u32,
}

#[derive(Resource)]
pub struct CosmosGenerationSettings {
    pub seed: u64,
    pub galaxy_radius: Length,
    pub num_stars: Range<u32>,
}

pub fn generate_cosmos(
    mut commands: Commands,
    mut signal: ResMut<InitializationSignal>,
    star_props: Res<CosmosStarPropertiesConfig>,
    star_names: Res<CosmosStarNamesConfig>,
    settings: Res<CosmosGenerationSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut star_materials: ResMut<Assets<StarMaterial>>,
    mut rocky_body_materials: ResMut<Assets<RockyBodyMaterial>>,
    mut giant_body_materials: ResMut<Assets<GiantBodyMaterial>>,
    mut orbit_materials: ResMut<Assets<OrbitMaterial>>,
) {
    if signal.cosmos_initialized {
        return;
    }

    signal.cosmos_initialized = true;

    info!("Start generating cosmos...");

    let start = Instant::now();

    info!("Start generating bodies...");

    let mut rng = StdRng::seed_from_u64(settings.seed);

    let mut stars = generate_stars(&mut rng, &settings, &star_props, &star_names);
    for star in stars.iter_mut() {
        let mut planets = generate_planets(&mut rng, &star);
        for planet in planets.iter_mut() {
            planet.children = generate_moon(&mut rng, planet);
        }
        star.children = planets;
    }

    convert_units(&mut stars);

    info!("Start placing bodies...");

    place_stars(&mut rng, &settings, &mut stars);

    stars.iter_mut().enumerate().for_each(|(i_star, star)| {
        place_planets(&mut rng, star, i_star);

        star.children
            .iter_mut()
            .enumerate()
            .for_each(|(i_planet, planet)| place_moons(&mut rng, planet, i_planet));
    });

    info!("Start generating orbits...");

    let (orbits, orbit_materials) = generate_orbits(&mut rng, &mut stars, &mut orbit_materials);

    info!("Start spawning all bodies and orbits into game...");

    // Shapes are implemented in shader, so we only need a square here.
    let square_mesh = Mesh2dHandle(meshes.add(Rectangle::from_length(1.)));

    let (bodies, entities, statistics) = spawn_bodies(
        &mut commands,
        // stars,
        vec![],
        square_mesh.clone(),
        &mut star_materials,
        &mut rocky_body_materials,
        &mut giant_body_materials,
    );

    spawn_orbits(&mut commands, square_mesh.clone(), &orbits, orbit_materials);

    commands.insert_resource(Cosmos {
        bodies,
        entities,
        orbits,
    });
    commands.insert_resource(GlobalRng::new(rng));
    commands.insert_resource(Ticker::default());

    info!(
        "Cosmos generation finished after {} s! {} star(s), {} planet(s), {} moon(s).",
        start.elapsed().as_secs_f32(),
        statistics.num_stars,
        statistics.num_planets,
        statistics.num_moons
    );
}

fn generate_stars(
    rng: &mut impl Rng,
    settings: &CosmosGenerationSettings,
    star_props: &CosmosStarPropertiesConfig,
    star_names: &CosmosStarNamesConfig,
) -> Vec<StarData> {
    let n = rng.gen_range(settings.num_stars.clone());
    let masses = reject_sampling(rng, star_mass_pdf, 0f64..130f64, 0f64..1f64, n, n * 2);

    let mut available_names = (**star_names).clone().into_iter().collect::<IndexSet<_>>();
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
        let radius = map_star_radius(floor.radius.lerp(ceil.radius, rng.gen_range(0f64..1f64)));
        let luminosity = floor
            .luminosity
            .lerp(ceil.luminosity, rng.gen_range(0f64..1f64));

        let name = available_names
            .swap_remove_index(rng.gen_range(0..available_names.len()))
            .unwrap();

        stars.push(StarData {
            name,
            body: CelestialBodyData {
                pos: Default::default(),
                mass,
                radius,
            },
            class: floor.class,
            luminosity,
            color: floor.color.lerp(ceil.color, rng.gen_range(0f32..1f32)),
            children: Vec::new(),
        });
    }

    stars
}

fn generate_planets(rng: &mut impl Rng, star: &StarData) -> Vec<PlanetData> {
    let n = rng.gen_range(1..=max_num_planets(star.class.index));

    let masses = reject_sampling(rng, planet_mass_pdf, 0.02f64..300f64, 0f64..2f64, n, n * 2);

    let rocky_density_distr = Normal::<f64>::new(3.5, 0.6).unwrap();
    let giant_density_distr = Normal::<f64>::new(1.2, 0.2).unwrap();

    let mut planets = Vec::with_capacity(n as usize);

    // Unit: Earth mass
    for mass in masses {
        let (density, ty) = {
            if mass > 100. {
                let density = sample_normal(rng, giant_density_distr, [0.4, 1.8]);
                (
                    density,
                    if density > 1.35 {
                        BodyType::IceGiant
                    } else {
                        BodyType::GasGiant
                    },
                )
            } else {
                (
                    sample_normal(rng, rocky_density_distr, [2.5, 5.]),
                    BodyType::Rocky,
                )
            }
        };

        let radius =
            map_planet_radius((Mass::EarthMass(mass).to_si() / density * 0.75 / PI).cbrt());

        planets.push(PlanetData {
            body: CelestialBodyData {
                pos: Default::default(),
                mass,
                radius,
            },
            orbit: Default::default(),
            effective_temp: 0.,
            color: random_color(rng),
            ty,
            children: Vec::new(),
        });
    }

    planets
}

fn generate_moon(rng: &mut impl Rng, planet: &PlanetData) -> Vec<MoonData> {
    let n = rng.gen_range(0..=max_num_moons(planet.body.mass));

    let masses = reject_sampling(rng, moon_mass_pdf, 0f64..1f64, 0f64..1f64, n, n * 2);
    let density = (0..n)
        .map(|_| sample_normal(rng, Normal::<f64>::new(2.5, 0.6).unwrap(), [1.2, 5.]))
        .collect::<Vec<_>>();

    let mut moons = Vec::with_capacity(n as usize);

    // Unit: Earth mass
    for (mass, density) in masses.into_iter().zip(density) {
        let radius = map_moon_radius((Mass::EarthMass(mass).to_si() / density * 0.75 / PI).cbrt());

        moons.push(MoonData {
            body: CelestialBodyData {
                pos: Default::default(),
                mass,
                radius,
            },
            orbit: Default::default(),
            effective_temp: 0.,
            color: random_color(rng),
        });
    }

    moons
}

fn convert_units(stars: &mut Vec<StarData>) {
    for star in stars {
        star.body.mass = Mass::SolarMass(star.body.mass).to_si();
        star.body.radius = Length::SolarRadius(star.body.radius).to_si();
        star.luminosity = RadiantFlux::SolarLuminosity(star.luminosity).to_si();

        for planet in &mut star.children {
            planet.body.mass = Mass::EarthMass(planet.body.mass).to_si();

            for moon in &mut planet.children {
                moon.body.mass = Mass::EarthMass(moon.body.mass).to_si();
            }
        }
    }
}

fn place_stars(rng: &mut impl Rng, settings: &CosmosGenerationSettings, stars: &mut Vec<StarData>) {
    let r_galaxy = settings.galaxy_radius.to_si();
    let theta = rng.gen_range(0.0..TAU);
    let square_r = reject_sampling(
        rng,
        star_pos_distr(r_galaxy * r_galaxy),
        0.0..r_galaxy * r_galaxy,
        0.0..1.0,
        stars.len() as u32,
        stars.len() as u32 * 5,
    );

    stars.iter_mut().zip(square_r).for_each(|(star, square_r)| {
        // star.body.pos = math::polar_to_cartesian(theta, square_r.sqrt());
    });
}

fn place_planets(rng: &mut impl Rng, star: &mut StarData, star_index: usize) {
    let mut cur_planet = 0;

    // Calculate boundaries
    let farthest = physics::dist_when_cycle(star.body.mass, 10800.);
    let closest = physics::dist_when_cycle(star.body.mass, 300.).max(star.body.radius * 5.);

    // --- Planets inside CHZ ---
    // Circumstellar Habitable Zone
    let chz_near = physics::planet_dist_when_temp(star.luminosity, 400., 0.5)
        .clamp(closest * 1.2, farthest * 0.8);
    let chz_far = physics::planet_dist_when_temp(star.luminosity, 200., 0.5)
        .clamp(closest * 1.2, farthest * 0.8);

    let n_chz = rng.sample(Normal::<f64>::new(0., 0.8).unwrap()).round() as usize;

    cur_planet += scatter_bodies_in_range(
        rng,
        star.children
            .iter_mut()
            .take(n_chz)
            .map(|p| &mut p.body)
            .collect(),
        [chz_near, chz_far],
        10,
    );

    let proportion = rng.gen_range(0.2..0.8);
    let remaining_planets = star.children.len() as u32 - cur_planet;

    // --- Planets too close to star ---

    let n_too_close = (proportion * remaining_planets as f64).round() as u32;

    cur_planet += scatter_bodies_in_range(
        rng,
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

    dbg!(
        star.body.radius,
        cur_planet,
        closest,
        chz_near,
        chz_far,
        farthest,
        n_too_close,
        n_chz,
        n_too_far
    );
}

fn place_moons(rng: &mut impl Rng, planet: &mut PlanetData, planet_index: usize) {
    let closest = physics::dist_when_cycle(planet.body.mass, 600.).max(planet.body.radius * 2.);
    let farthest = physics::dist_when_cycle(planet.body.mass, 3600.);

    let succeeded = scatter_bodies_in_range(
        rng,
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
    mut bodies: Vec<&mut CelestialBodyData>,
    boundaries: [f64; 2],
    max_failure: u32,
) -> u32 {
    if boundaries[0] > boundaries[1] {
        return 0;
    }

    let mut cur_dist = boundaries[0];
    let mut cur_body = 0;
    let mut cur_failed = 0;

    while cur_body < bodies.len() {
        let t = rng.gen_range(0f64..1f64 / bodies.len() as f64);
        let delta = (boundaries[1] - boundaries[0]) * t;
        if cur_dist + delta < boundaries[1] {
            bodies[cur_body].pos = DVec2::new(cur_dist + delta, 0.);

            cur_body += 1;
            cur_dist += delta;
        } else {
            cur_failed += 1;
        }

        if cur_failed >= max_failure {
            break;
        }
    }

    cur_body as u32
}

fn generate_orbits(
    rng: &mut impl Rng,
    stars: &mut Vec<StarData>,
    orbit_material_assets: &mut Assets<OrbitMaterial>,
) -> (Vec<Orbit>, Vec<Option<Handle<OrbitMaterial>>>) {
    // Estimate the number of orbits roughly equal to star.len() * 5
    let mut orbits = Vec::with_capacity(stars.len() * 5);
    let mut orbit_materials = Vec::with_capacity(stars.len() * 5);
    let mut cur_body = 0;

    for star in stars {
        let i_star = cur_body;
        orbits.push(Orbit {
            center_id: usize::MAX,
            ..Default::default()
        });
        orbit_materials.push(None);
        cur_body += 1;

        // for planet in &mut star.children {
        //     let distance = planet.body.pos.length();
        //     let i_planet = cur_body;
        //     orbits.push(Orbit {
        //         initial_progress: rng.gen_range(0f64..TAU),
        //         center_id: i_star,
        //         center: Default::default(),
        //         radius: distance,
        //         sidereal_period: Time::Second(
        //             (TAU / physics::angular_vel_between(star.body.mass, distance)) as u64,
        //         )
        //         .to_si(),
        //         rotation_period: Time::Second(rng.gen_range(100..1800)).to_si(),
        //     });
        //     orbit_materials.push(Some(orbit_material_assets.add(OrbitMaterial {
        //         color: planet.color.with_alpha(0.5),
        //         width: ORBIT_WIDTH,
        //         radius: distance as f32,
        //     })));
        //     cur_body += 1;

        //     for moon in &mut planet.children {
        //         let distance = moon.body.pos.length();
        //         orbits.push(Orbit {
        //             initial_progress: rng.gen_range(0f64..TAU),
        //             center_id: i_planet,
        //             center: Default::default(),
        //             radius: distance,
        //             sidereal_period: Time::Second(
        //                 (TAU / physics::angular_vel_between(planet.body.mass, distance)) as u64,
        //             )
        //             .to_si(),
        //             rotation_period: Time::Second(rng.gen_range(100..1800)).to_si(),
        //         });
        //         orbit_materials.push(Some(orbit_material_assets.add(OrbitMaterial {
        //             color: moon.color.with_alpha(0.5),
        //             width: ORBIT_WIDTH,
        //             radius: distance as f32,
        //         })));
        //         cur_body += 1;
        //     }
        // }
    }

    (orbits, orbit_materials)
}

fn spawn_bodies(
    commands: &mut Commands,
    stars: Vec<StarData>,
    mesh: Mesh2dHandle,
    star_materials: &mut Assets<StarMaterial>,
    rocky_body_materials: &mut Assets<RockyBodyMaterial>,
    giant_body_materials: &mut Assets<GiantBodyMaterial>,
) -> (Vec<CelestialBodyData>, Vec<Entity>, CosmosBodiesStatistics) {
    // Estimate the number of bodies roughly equal to star.len() * 5
    let mut bodies = Vec::with_capacity(stars.len() * 5);
    let mut entities = Vec::with_capacity(stars.len() * 5);
    let mut statistics = CosmosBodiesStatistics::default();

    for star in stars {
        entities.push(
            commands
                .spawn(StarBundle {
                    star_ty: star.class.ty,
                    name: Name::new(star.name.clone()),
                    body_index: BodyIndex::new(bodies.len()),
                    mesh: mesh.clone(),
                    material: star_materials.add(StarMaterial { color: star.color }),
                    transform: Transform::from_scale(Vec3::splat(star.body.radius as f32 * 2.)),
                    collider: Collider::circle(0.5),
                    ..Default::default()
                })
                .id(),
        );
        statistics.num_stars += 1;
        bodies.push(star.body);

        let mut i_children = 0;

        // for planet in star.children {
        //     if planet.ty == BodyType::Rocky {
        //         entities.push(
        //             commands
        //                 .spawn((
        //                     RockyBodyBundle {
        //                         name: Name::new(format!("{} {}", &star.name, i_children)),
        //                         ty: planet.ty,
        //                         body_index: BodyIndex::new(bodies.len()),
        //                         mesh: mesh.clone(),
        //                         material: rocky_body_materials.add(RockyBodyMaterial {
        //                             color: planet.color,
        //                         }),
        //                         transform: Transform::from_scale(Vec3::splat(
        //                             planet.body.radius as f32 * 2.,
        //                         )),
        //                         collider: Collider::circle(0.5),
        //                         ..Default::default()
        //                     },
        //                     Planet,
        //                 ))
        //                 .id(),
        //         );
        //     } else {
        //         entities.push(
        //             commands
        //                 .spawn((
        //                     GiantBodyBundle {
        //                         name: Name::new(format!("{} {}", &star.name, i_children)),
        //                         ty: planet.ty,
        //                         body_index: BodyIndex::new(bodies.len()),
        //                         mesh: mesh.clone(),
        //                         material: giant_body_materials.add(GiantBodyMaterial {
        //                             color: planet.color,
        //                         }),
        //                         transform: Transform::from_scale(Vec3::splat(
        //                             planet.body.radius as f32 * 2.,
        //                         )),
        //                         collider: Collider::circle(0.5),
        //                         ..Default::default()
        //                     },
        //                     Planet,
        //                 ))
        //                 .id(),
        //         );
        //     }

        //     statistics.num_planets += 1;
        //     i_children += 1;
        //     bodies.push(planet.body);

        //     for moon in planet.children {
        //         entities.push(
        //             commands
        //                 .spawn((
        //                     RockyBodyBundle {
        //                         name: Name::new(format!("{} {}", star.name, i_children)),
        //                         ty: BodyType::Rocky,
        //                         body_index: BodyIndex::new(bodies.len()),
        //                         mesh: mesh.clone(),
        //                         material: rocky_body_materials
        //                             .add(RockyBodyMaterial { color: moon.color }),
        //                         transform: Transform::from_scale(Vec3::splat(
        //                             moon.body.radius as f32 * 2.,
        //                         )),
        //                         collider: Collider::circle(0.5),
        //                         ..Default::default()
        //                     },
        //                     Moon,
        //                 ))
        //                 .id(),
        //         );

        //         statistics.num_moons += 1;
        //         i_children += 1;
        //         bodies.push(moon.body);
        //     }
        // }
    }

    (bodies, entities, statistics)
}

fn spawn_orbits(
    commands: &mut Commands,
    square_mesh: Mesh2dHandle,
    orbits: &Vec<Orbit>,
    orbit_materials: Vec<Option<Handle<OrbitMaterial>>>,
) {
    for (i_orbit, (material, orbit)) in orbit_materials.into_iter().zip(orbits).enumerate() {
        let Some(material) = material else {
            continue;
        };

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: square_mesh.clone(),
                material,
                transform: Transform::from_scale(Vec3::splat(
                    orbit.radius as f32 * 2. * ORBIT_MESH_SCALE,
                )),
                ..Default::default()
            },
            OrbitIndex::new(i_orbit),
        ));
    }
}

fn random_color(rng: &mut impl Rng) -> LinearRgba {
    LinearRgba {
        red: rng.gen_range(0.0..1.0),
        green: rng.gen_range(0.0..1.0),
        blue: rng.gen_range(0.0..1.0),
        alpha: rng.gen_range(0.0..1.0),
    }
}

fn sample_normal<T>(rng: &mut impl Rng, distr: Normal<T>, min_max: [T; 2]) -> T
where
    T: Float,
    StandardNormal: Distribution<T>,
{
    loop {
        let t = rng.sample(distr);
        if t > min_max[0] && t < min_max[1] {
            return t;
        }
    }
}

fn map_star_radius(x: f64) -> f64 {
    (-x / 500. + 1.).powf(1.6)
}

fn map_planet_radius(x: f64) -> f64 {
    x * 0.02
}

fn map_moon_radius(x: f64) -> f64 {
    x * 0.02
}
