use bevy::{
    log::warn,
    math::{Vec2, Vec3, Vec3Swizzles},
    prelude::{
        Camera, Commands, Component, Entity, GlobalTransform, Query, Res, Transform,
        ViewVisibility, With, Without,
    },
    ui::{Node, Style, Val},
};

use crate::{
    math::Direction,
    sim::{CursorPosition, MainCamera, ViewScale, WindowSize},
    ui::UiPos,
};

#[derive(Default, Clone)]
pub struct SyncWhenInvisibleOptions {
    pub sync_when_invisible: bool,
    pub keep_component_when_invisible: bool,
}

bitflags::bitflags! {
    pub struct UiSyncFilter: u32 {
        const TRANSLATION = 1 << 0;
        const ROTATION    = 1 << 1;
        const SCALE       = 1 << 2;
    }
}

pub enum ScaleMethod {
    Direct,
    Calculated { initial_elem_size: Option<Vec2> },
}

#[derive(Component)]
pub struct UiSyncWithSceneEntity {
    pub target: Entity,
    pub scene_offset: Vec3,
    pub ui_offset: [Val; 2],
    pub axes_filter: Vec2,
    pub filter: UiSyncFilter,
    pub scale_method: Option<ScaleMethod>,
    pub invis: SyncWhenInvisibleOptions,
}

impl Default for UiSyncWithSceneEntity {
    fn default() -> Self {
        Self {
            target: Entity::PLACEHOLDER,
            scene_offset: Default::default(),
            ui_offset: Default::default(),
            axes_filter: Vec2::ONE,
            filter: UiSyncFilter::TRANSLATION,
            scale_method: None,
            invis: Default::default(),
        }
    }
}

#[derive(Component)]
pub struct UiSyncWithCursor {
    pub offset: Vec2,
    pub axes_filter: Vec2,
    pub initial_cursor_pos: Vec2,
    pub initial_elem_pos: Option<UiPos>,
    pub invis: SyncWhenInvisibleOptions,
}

impl Default for UiSyncWithCursor {
    fn default() -> Self {
        Self {
            offset: Default::default(),
            axes_filter: Vec2::ONE,
            initial_cursor_pos: Default::default(),
            initial_elem_pos: Default::default(),
            invis: Default::default(),
        }
    }
}

pub fn scene_ui_sync(
    mut commands: Commands,
    scene_entities_query: Query<&Transform, Without<Node>>,
    mut ui_query: Query<(
        Entity,
        &mut UiSyncWithSceneEntity,
        &Node,
        &mut Style,
        &mut Transform,
        &ViewVisibility,
    )>,
    main_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    window_size: Res<WindowSize>,
    view_scale: Res<ViewScale>,
) {
    let (main_camera, camera_transform) = main_camera.single();

    for (entity, mut sync, node, mut style, mut transform, vis) in &mut ui_query {
        if !vis.get() {
            if !sync.invis.sync_when_invisible {
                if !sync.invis.keep_component_when_invisible {
                    commands.entity(entity).remove::<UiSyncWithSceneEntity>();
                }
                continue;
            }
        }

        let (Ok(target), Ok(ui_pos)) = (scene_entities_query.get(sync.target), UiPos::new(&style))
        else {
            continue;
        };

        if sync.filter.contains(UiSyncFilter::TRANSLATION) {
            scene_ui_sync_translation(
                main_camera,
                camera_transform,
                &*sync,
                node,
                target,
                **window_size,
                ui_pos,
                &mut style,
            );
        }

        if sync.filter.contains(UiSyncFilter::ROTATION) {
            scene_ui_sync_rotation(&mut transform, target);
        }

        if sync.filter.contains(UiSyncFilter::SCALE) {
            scene_ui_sync_scale(
                &mut transform,
                &mut style,
                **view_scale,
                target,
                &mut sync.scale_method,
            );
        }
    }
}

fn scene_ui_sync_translation(
    main_camera: &Camera,
    camera_transform: &GlobalTransform,
    sync: &UiSyncWithSceneEntity,
    node: &Node,
    target: &Transform,
    window_size: Vec2,
    mut ui_pos: UiPos,
    style: &mut Style,
) {
    let Some(mut scene_entity_viewport_pos) =
        main_camera.world_to_viewport(camera_transform, target.translation + sync.scene_offset)
    else {
        return;
    };

    let Vec2 {
        x: node_x,
        y: node_y,
    } = node.size();
    scene_entity_viewport_pos += Vec2 {
        x: sync.ui_offset[0].resolve(node_x, window_size).unwrap(),
        y: sync.ui_offset[1].resolve(node_y, window_size).unwrap(),
    };

    if ui_pos.dirs[0] != Direction::Left {
        scene_entity_viewport_pos.x = window_size.x - scene_entity_viewport_pos.x;
    }
    if ui_pos.dirs[1] != Direction::Up {
        scene_entity_viewport_pos.y = window_size.y - scene_entity_viewport_pos.y;
    }

    ui_pos.pos = scene_entity_viewport_pos * sync.axes_filter;
    ui_pos.apply_to(style);
}

fn scene_ui_sync_rotation(ui: &mut Transform, target: &Transform) {
    ui.rotation = target.rotation;
}

fn scene_ui_sync_scale(
    ui: &mut Transform,
    style: &mut Style,
    view_scale: f32,
    target: &Transform,
    method: &mut Option<ScaleMethod>,
) {
    let Some(method) = method else {
        return;
    };

    match method {
        ScaleMethod::Direct => ui.scale = target.scale / view_scale,
        ScaleMethod::Calculated { initial_elem_size } => {
            if initial_elem_size.is_none() {
                *initial_elem_size = Some(Vec2 {
                    x: match style.width {
                        Val::Px(px) => px,
                        _ => return,
                    },
                    y: match style.height {
                        Val::Px(px) => px,
                        _ => return,
                    },
                });
            }

            let size = initial_elem_size.unwrap() * target.scale.xy() / view_scale;
            style.width = Val::Px(size.x);
            style.height = Val::Px(size.y);
        }
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

        let offset = (cursor_pos - sync.initial_cursor_pos + sync.offset) * sync.axes_filter;

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
