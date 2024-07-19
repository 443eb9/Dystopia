use std::f64::consts::TAU;

use bevy::{
    math::Vec3,
    prelude::{DetectChanges, Query, Res, ResMut},
    render::view::Visibility,
    transform::components::Transform,
};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::{
    cosmos::celestial::{BodyIndex, Cosmos, OrbitIndex, ShowOrbits},
    math,
    simulation::{Ticker, ViewScale},
};

pub fn update_cosmos(mut cosmos: ResMut<Cosmos>, ticker: Res<Ticker>) {
    let Cosmos { bodies, orbits } = cosmos.as_mut();

    bodies
        .par_iter_mut()
        .zip(orbits.par_iter_mut())
        .for_each(|(body, orbit)| {
            if orbit.radius < 0. {
                return;
            }

            let progress =
                (ticker.0 as f64 / orbit.sidereal_period as f64 + orbit.initial_progress).fract();
            body.pos = orbit.center + math::polar_to_cartesian(progress * TAU, orbit.radius);
        });

    orbits.par_iter_mut().for_each(|orbit| {
        if orbit.radius < 0. {
            return;
        }
        orbit.center = bodies[orbit.center_id].pos;
    });
}

pub fn sync_bodies(cosmos: Res<Cosmos>, mut bodies_query: Query<(&BodyIndex, &mut Transform)>) {
    bodies_query
        .par_iter_mut()
        .for_each(|(i_body, mut transform)| {
            let body = &cosmos.bodies[i_body.0];
            transform.translation = Vec3 {
                x: body.pos.x as f32,
                y: body.pos.y as f32,
                z: 0.,
            };
        });
}

pub fn sync_orbits(
    cosmos: Res<Cosmos>,
    show_orbits: Res<ShowOrbits>,
    mut orbits_query: Query<(&OrbitIndex, &mut Transform, &mut Visibility)>,
) {
    orbits_query
        .par_iter_mut()
        .for_each(|(i_orbit, mut transform, mut visibility)| {
            let orbit = &cosmos.orbits[i_orbit.0];
            transform.translation = Vec3 {
                x: orbit.center.x as f32,
                y: orbit.center.y as f32,
                z: 0.,
            };

            *visibility = {
                if *show_orbits.get() {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                }
            };
        });
}

pub fn manage_orbit_visibility(view_scale: Res<ViewScale>, mut show_orbits: ResMut<ShowOrbits>) {
    if view_scale.is_changed() {
        show_orbits.set(*view_scale.get() < 1.);
    }
}
