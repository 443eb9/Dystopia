use std::f64::consts::TAU;

use bevy::{
    math::Vec3,
    prelude::{Query, Res, ResMut},
    transform::components::Transform,
};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::{
    cosmos::celestial::{BodyIndex, Cosmos},
    math,
    simulation::Ticker,
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
