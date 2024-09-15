use avian2d::prelude::Collider;
use bevy::{
    asset::Handle,
    core::Name,
    prelude::Bundle,
    render::view::{InheritedVisibility, ViewVisibility, Visibility},
    sprite::Mesh2dHandle,
    transform::components::{GlobalTransform, Transform},
};

use crate::{
    cosmos::{
        celestial::{BodyColor, BodyIndex, BodyType, Star, StarType, System},
        mesh::{GiantBodyMaterial, RockyBodyMaterial, StarMaterial},
    },
    scene::transition::CameraRecoverTransform,
};

#[derive(Bundle, Default)]
pub struct StarBundle {
    pub star: Star,
    pub star_ty: StarType,
    pub name: Name,
    pub body_index: BodyIndex,
    pub system: System,
    pub mesh: Mesh2dHandle,
    pub material: Handle<StarMaterial>,
    pub color: BodyColor,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub recover_position: CameraRecoverTransform,
    pub collider: Collider,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

#[derive(Bundle, Default)]
pub struct RockyBodyBundle {
    pub name: Name,
    pub ty: BodyType,
    pub body_index: BodyIndex,
    pub system: System,
    pub mesh: Mesh2dHandle,
    pub material: Handle<RockyBodyMaterial>,
    pub color: BodyColor,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub recover_position: CameraRecoverTransform,
    pub collider: Collider,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

#[derive(Bundle, Default)]
pub struct GiantBodyBundle {
    pub name: Name,
    pub ty: BodyType,
    pub body_index: BodyIndex,
    pub system: System,
    pub mesh: Mesh2dHandle,
    pub material: Handle<GiantBodyMaterial>,
    pub color: BodyColor,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub recover_position: CameraRecoverTransform,
    pub collider: Collider,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}
