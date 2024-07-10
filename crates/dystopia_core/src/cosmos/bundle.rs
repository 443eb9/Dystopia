use bevy::{
    asset::Handle,
    prelude::Bundle,
    render::view::{InheritedVisibility, ViewVisibility, Visibility},
    sprite::Mesh2dHandle,
    transform::components::{GlobalTransform, Transform},
};

use crate::cosmos::{
    celestial::{BodyIndex, Planet, Star},
    mesh::{GiantBodyMaterial, RockyBodyMaterial, StarMaterial},
};

#[derive(Bundle, Default)]
pub struct StarBundle {
    pub star: Star,
    pub body_index: BodyIndex,
    pub mesh: Mesh2dHandle,
    pub material: Handle<StarMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

#[derive(Bundle, Default)]
pub struct RockyBodyBundle {
    pub planet: Planet,
    pub body_index: BodyIndex,
    pub mesh: Mesh2dHandle,
    pub material: Handle<RockyBodyMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

#[derive(Bundle, Default)]
pub struct GiantBodyBundle {
    pub planet: Planet,
    pub body_index: BodyIndex,
    pub mesh: Mesh2dHandle,
    pub material: Handle<GiantBodyMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}
