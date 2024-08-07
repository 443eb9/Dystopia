use bevy::{
    log::warn,
    math::{Vec2, Vec3},
    prelude::{
        Camera, Commands, Component, Entity, GlobalTransform, Query, Res, Transform,
        ViewVisibility, With,
    },
    ui::{Node, Style, Val},
};

use crate::{
    math::Direction,
    simulation::{CursorPosition, MainCamera, ViewScale, WindowSize},
    ui::UiPos,
};

#[derive(Default, Clone)]
pub struct SyncWhenInvisibleOptions {
    pub sync_when_invisible: bool,
    pub keep_component_when_invisible: bool,
}

#[derive(Component)]
pub struct UiSyncWithSceneEntity {
    pub target: Entity,
    pub scene_offset: Vec3,
    pub ui_offset: [Val; 2],
    pub filter: Vec2,
    pub invis: SyncWhenInvisibleOptions,
}

impl Default for UiSyncWithSceneEntity {
    fn default() -> Self {
        Self {
            target: Entity::PLACEHOLDER,
            scene_offset: Default::default(),
            ui_offset: Default::default(),
            filter: Vec2::ONE,
            invis: Default::default(),
        }
    }
}

#[derive(Component)]
pub struct UiSyncCameraScaleWithSceneEntity {
    pub target: Entity,
    pub initial_view_scale: Option<f32>,
    pub initial_elem_size: Option<Vec2>,
    pub invis: SyncWhenInvisibleOptions,
}

impl Default for UiSyncCameraScaleWithSceneEntity {
    fn default() -> Self {
        Self {
            target: Entity::PLACEHOLDER,
            initial_view_scale: Default::default(),
            initial_elem_size: Default::default(),
            invis: Default::default(),
        }
    }
}

#[derive(Component)]
pub struct UiSyncWithCursor {
    pub offset: Vec2,
    pub filter: Vec2,
    pub initial_cursor_pos: Vec2,
    pub initial_elem_pos: Option<UiPos>,
    pub invis: SyncWhenInvisibleOptions,
}

impl Default for UiSyncWithCursor {
    fn default() -> Self {
        Self {
            offset: Default::default(),
            filter: Vec2::ONE,
            initial_cursor_pos: Default::default(),
            initial_elem_pos: Default::default(),
            invis: Default::default(),
        }
    }
}

pub fn scene_ui_sync_tranlation(
    mut commands: Commands,
    scene_entities_query: Query<&Transform>,
    mut ui_query: Query<(
        Entity,
        &UiSyncWithSceneEntity,
        &Node,
        &mut Style,
        &ViewVisibility,
    )>,
    main_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    window_size: Res<WindowSize>,
) {
    let (main_camera, camera_transform) = main_camera.single();

    for (entity, sync, node, mut style, vis) in &mut ui_query {
        if !vis.get() {
            if !sync.invis.sync_when_invisible {
                if !sync.invis.keep_component_when_invisible {
                    commands.entity(entity).remove::<UiSyncWithSceneEntity>();
                }
                continue;
            }
        }

        let (Ok(target), Ok(mut ui_pos)) =
            (scene_entities_query.get(sync.target), UiPos::new(&style))
        else {
            continue;
        };

        let Some(mut scene_entity_viewport_pos) =
            main_camera.world_to_viewport(camera_transform, target.translation + sync.scene_offset)
        else {
            continue;
        };

        let Vec2 {
            x: node_x,
            y: node_y,
        } = node.size();
        scene_entity_viewport_pos += Vec2 {
            x: sync.ui_offset[0].resolve(node_x, **window_size).unwrap(),
            y: sync.ui_offset[1].resolve(node_y, **window_size).unwrap(),
        };

        if ui_pos.dirs[0] != Direction::Left {
            scene_entity_viewport_pos.x = window_size.x - scene_entity_viewport_pos.x;
        }
        if ui_pos.dirs[1] != Direction::Up {
            scene_entity_viewport_pos.y = window_size.y - scene_entity_viewport_pos.y;
        }

        ui_pos.pos = scene_entity_viewport_pos * sync.filter;
        ui_pos.apply_to(&mut style);
    }
}

pub fn scene_ui_sync_camera_scale(
    mut commands: Commands,
    mut ui_query: Query<(
        Entity,
        &mut UiSyncCameraScaleWithSceneEntity,
        &mut Style,
        &ViewVisibility,
    )>,
    view_scale: Res<ViewScale>,
) {
    for (entity, mut sync, mut style, vis) in &mut ui_query {
        if !vis.get() {
            if !sync.invis.sync_when_invisible {
                if !sync.invis.keep_component_when_invisible {
                    commands
                        .entity(entity)
                        .remove::<UiSyncCameraScaleWithSceneEntity>();
                }
                continue;
            }
        }

        if sync.initial_elem_size.is_none() {
            sync.initial_elem_size = Some(Vec2 {
                x: match style.width {
                    Val::Px(px) => px,
                    _ => continue,
                },
                y: match style.height {
                    Val::Px(px) => px,
                    _ => continue,
                },
            });
        }

        if sync.initial_view_scale.is_none() {
            sync.initial_view_scale = Some(**view_scale);
        }

        let size =
            sync.initial_elem_size.unwrap() * sync.initial_view_scale.unwrap() / **view_scale;
        style.width = Val::Px(size.x);
        style.height = Val::Px(size.y);
    }
}

pub fn cursor_ui_sync(
    mut commands: Commands,
    mut ui_query: Query<(Entity, &mut UiSyncWithCursor, &mut Style, &ViewVisibility)>,
    cursor_pos: Res<CursorPosition>,
) {
    let Some(cursor_pos) = **cursor_pos else {
        return;
    };

    for (entity, mut sync, mut style, vis) in &mut ui_query {
        if !vis.get() {
            if !sync.invis.sync_when_invisible {
                if !sync.invis.keep_component_when_invisible {
                    commands.entity(entity).remove::<UiSyncWithCursor>();
                }
                continue;
            }
        }

        let offset = (cursor_pos - sync.initial_cursor_pos + sync.offset) * sync.filter;

        if sync.initial_elem_pos.is_none() {
            match UiPos::new(&style) {
                Ok(ok) => sync.initial_elem_pos = Some(ok),
                Err(err) => {
                    warn!("Failed to create UiPos: {:?}", err);
                    continue;
                }
            }
        }

        (sync.initial_elem_pos.unwrap() + offset).apply_to(&mut style);
    }
}
