use bevy::{
    app::{App, Plugin},
    math::{Quat, Vec3},
    prelude::{Component, Resource, Transform},
};

use crate::sim::ViewScale;

pub mod cosmos_view;
pub mod focusing_body;

pub struct DystopiaTransitionPlugin;

impl Plugin for DystopiaTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            cosmos_view::CosmosViewSceneTransitionPlugin,
            focusing_body::FocusingBodyTransitionPlugin,
        ))
        .init_resource::<CameraRecoverTransform>();
    }
}

/// When player focuses a body, entering cosmos view, etc, the camera will try to
/// recover the view when last defocus, etc.
///
/// Can also be used as a resource.
#[derive(Resource, Component, Debug)]
pub struct CameraRecoverTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: f32,
}

impl Default for CameraRecoverTransform {
    fn default() -> Self {
        Self {
            translation: Default::default(),
            rotation: Default::default(),
            scale: 1.,
        }
    }
}

impl CameraRecoverTransform {
    #[inline]
    pub fn new(transform: &Transform, scale: &ViewScale) -> Self {
        Self {
            translation: transform.translation,
            rotation: transform.rotation,
            scale: **scale,
        }
    }

    #[inline]
    pub fn update(&mut self, transform: &Transform, scale: &ViewScale) {
        self.translation = transform.translation;
        self.rotation = transform.rotation;
        self.scale = **scale;
    }

    #[inline]
    pub fn recover(&self, transform: &mut Transform, scale: &mut ViewScale) {
        transform.translation = self.translation;
        transform.rotation = self.rotation;
        **scale = self.scale;
    }
}

#[macro_export]
macro_rules! impl_transition_plugin {
    ($ty: ident, $state: expr, $on_enter: tt, $on_update: tt, $on_exit: tt) => {
        pub struct $ty;

        impl bevy::prelude::Plugin for $ty {
            fn build(&self, app: &mut bevy::prelude::App) {
                use bevy::prelude::IntoSystemConfigs;

                app.add_systems(bevy::prelude::OnEnter($state), $on_enter)
                    .add_systems(
                        bevy::prelude::Update,
                        $on_update.run_if(bevy::prelude::in_state($state)),
                    )
                    .add_systems(bevy::prelude::OnExit($state), $on_exit);
            }
        }
    };

    ($ty: ident, $state: expr, $on_enter: tt, $on_update: expr, $on_exit: tt) => {
        impl_transition_plugin!($ty, $state, $on_enter, $on_update, $on_exit);
    };
}
