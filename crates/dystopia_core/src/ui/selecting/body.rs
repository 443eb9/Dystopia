use bevy::{
    asset::{Asset, Assets},
    color::{ColorToComponents, LinearRgba},
    math::{Vec2, Vec3},
    prelude::{
        Commands, Deref, Entity, EventReader, FromWorld, MaterialNodeBundle, Res, Resource,
        Visibility, World,
    },
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    ui::{UiMaterial, Val},
};

use crate::{
    input::RayTransparent,
    ui::{
        panel::{body_data::BodyDataPanel, PanelTargetChange},
        sync::{ScaleMethod, SyncWhenInvisibleOptions, UiSyncFilter, UiSyncWithSceneEntity},
    },
};

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

pub fn on_target_change(
    mut commands: Commands,
    mut target_change: EventReader<PanelTargetChange<BodyDataPanel>>,
    indicator: Res<BodySelectingIndicator>,
) {
    for change in target_change.read() {
        match **change {
            // TODO indicator flashes when switching between bodies
            Some(target) => commands.entity(**indicator).insert((
                UiSyncWithSceneEntity {
                    target,
                    ui_offset: [Val::Percent(-50.), Val::Percent(-50.)],
                    filter: UiSyncFilter::TRANSLATION | UiSyncFilter::SCALE,
                    scale_method: Some(ScaleMethod::Calculated {
                        initial_elem_size: Some(Vec2::splat(2.)),
                    }),
                    invis: SyncWhenInvisibleOptions {
                        sync_when_invisible: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Visibility::Visible,
            )),
            None => commands.entity(**indicator).insert(Visibility::Hidden),
        };
    }
}
