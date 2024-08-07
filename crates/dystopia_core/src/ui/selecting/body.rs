use bevy::{
    asset::{Asset, Assets},
    color::{ColorToComponents, LinearRgba},
    math::Vec3,
    prelude::{Deref, Entity, FromWorld, MaterialNodeBundle, Resource, Visibility, World},
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    ui::UiMaterial,
};

use crate::input::RayTransparent;

#[derive(AsBindGroup, Asset, TypePath, Clone)]
#[uniform(0, BodySelectingIconMaterialUniform)]
pub struct BodySelectingIconMaterial {
    pub line_color: LinearRgba,
    pub line_width: f32,
}

impl UiMaterial for BodySelectingIconMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ui/selecting.wgsl".into()
    }
}

#[derive(ShaderType)]
pub struct BodySelectingIconMaterialUniform {
    pub line_color: Vec3,
    pub line_width: f32,
}

impl From<&BodySelectingIconMaterial> for BodySelectingIconMaterialUniform {
    fn from(value: &BodySelectingIconMaterial) -> Self {
        Self {
            line_color: value.line_color.to_vec3(),
            line_width: value.line_width,
        }
    }
}

#[derive(Resource, Deref)]
pub struct BodySelectingIndicator(Entity);

impl FromWorld for BodySelectingIndicator {
    fn from_world(world: &mut World) -> Self {
        let material = world
            .resource_mut::<Assets<BodySelectingIconMaterial>>()
            .add(BodySelectingIconMaterial {
                line_color: LinearRgba::WHITE,
                line_width: 5.,
            });

        let entity = world
            .spawn((
                MaterialNodeBundle {
                    material,
                    visibility: Visibility::Hidden,
                    ..Default::default()
                },
                RayTransparent,
            ))
            .id();

        Self(entity)
    }
}
