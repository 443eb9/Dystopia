use std::f64::consts::TAU;

use bevy::{
    asset::Assets,
    color::Alpha,
    math::{FloatExt, Vec3},
    prelude::{DetectChanges, Local, Query, Res, ResMut, With},
    render::view::Visibility,
    time::{Real, Time},
    transform::components::Transform,
};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::{
    body::FocusingOn,
    cosmos::{
        celestial::{BodyIndex, Cosmos, OrbitIndex, OrbitsVisibility},
        mesh::OrbitMaterial,
    },
    math,
    scene::transition::CameraRecoverTransform,
    sim::{MainCamera, Ticker, ViewScale},
};

pub fn update_cosmos(mut cosmos: ResMut<Cosmos>, ticker: Res<Ticker>) {
    let Cosmos {
        bodies,
        entities: _,
        orbits,
    } = cosmos.as_mut();

    bodies
        .par_iter_mut()
        .zip(orbits.par_iter_mut())
        .for_each(|(body, orbit)| {
            if orbit.center_id == usize::MAX {
                return;
            }

            let progress =
                (**ticker as f64 / orbit.sidereal_period as f64 + orbit.initial_progress).fract();
            body.pos = orbit.center + math::polar_to_cartesian(progress * TAU, orbit.radius);
        });

    orbits.par_iter_mut().for_each(|orbit| {
        if orbit.center_id == usize::MAX {
            return;
        }

        orbit.center = bodies[orbit.center_id].pos;
    });
}

pub fn sync_bodies(cosmos: Res<Cosmos>, mut bodies_query: Query<(&BodyIndex, &mut Transform)>) {
    bodies_query
        .par_iter_mut()
        .for_each(|(i_body, mut transform)| {
            let body = &cosmos.bodies[**i_body];
            transform.translation = Vec3 {
                x: body.pos.x as f32,
                y: body.pos.y as f32,
                z: 0.,
            };
        });
}

pub fn sync_orbits(cosmos: Res<Cosmos>, mut orbits_query: Query<(&OrbitIndex, &mut Transform)>) {
    orbits_query
        .par_iter_mut()
        .for_each(|(i_orbit, mut transform)| {
            let orbit = &cosmos.orbits[**i_orbit];
            transform.translation = Vec3 {
                x: orbit.center.x as f32,
                y: orbit.center.y as f32,
                z: 0.,
            };
        });
}

pub fn manage_orbit_visibility(
    view_scale: Res<ViewScale>,
    orbits_vis: Res<OrbitsVisibility>,
    mut orbit_materials: ResMut<Assets<OrbitMaterial>>,
    mut maybe_target_vis: Local<Option<bool>>,
    mut current_alpha: Local<f32>,
    time: Res<Time<Real>>,
) {
    if view_scale.is_changed() {
        let vis = **view_scale < orbits_vis.scale_threshold;
        if maybe_target_vis.is_none() {
            *maybe_target_vis = Some(vis);
        } else {
            if maybe_target_vis.is_some_and(|v| v != vis) {
                *maybe_target_vis = Some(vis);
            }
        }
    }

    let Some(target_vis) = *maybe_target_vis else {
        return;
    };

    *current_alpha = current_alpha.lerp(
        if target_vis { orbits_vis.alpha } else { 0. },
        orbits_vis.fade_speed * time.delta_seconds(),
    );

    // TODO use a uniform buffer to sync alpha. This is tooooo slow.
    orbit_materials
        .iter_mut()
        .for_each(|(_, mat)| mat.color = mat.color.with_alpha(*current_alpha));
}

pub fn sync_recover_position(
    fousing_on: Res<FocusingOn>,
    mut bodies_query: Query<&mut CameraRecoverTransform, With<BodyIndex>>,
    camera_query: Query<&Transform, With<MainCamera>>,
    view_scale: Res<ViewScale>,
) {
    bodies_query
        .get_mut(fousing_on.entity)
        .unwrap()
        .update(camera_query.single(), &view_scale);
}
